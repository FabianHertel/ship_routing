// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod dijkstra;
mod a_star;
mod bidirectional_dijkstra;
mod binary_minheap;
mod binary_minheap_map;
mod ch;
mod test_routing;
mod ws_a_star;

use graph_lib::{Coordinates, Graph, file_interface::import_graph_from_file };
use test_routing::{test_samples, test_random_samples_ch, test_random_samples_a_star, test_random_samples_dijkstra, test_random_samples_bd};

use crate::{bidirectional_dijkstra::run_bidirectional_dijkstra, ch::{new_ch_precalculations, run_ch, continue_ch_precalculations}, a_star::run_a_star, dijkstra::run_dijkstra};

static mut GRAPH: Option<Graph> = None;
static mut CH_GRAPH: Option<Graph> = None;
static mut ROUTING: Routing = Routing::CH;

pub enum Routing {
    DI, BD, ASTAR, CH
}

#[tauri::command]
fn route(coordinates: [[f32;2];2]) -> Vec<[f32;2]> {

    let src_coordinates = Coordinates(coordinates[0][1], coordinates[0][0]);
    let tgt_coordinates = Coordinates(coordinates[1][1], coordinates[1][0]);
    let mut shortest_path = Vec::new();
    
    unsafe {
        let (src_node, tgt_node) = match ROUTING {
            Routing::CH => (CH_GRAPH.as_ref().unwrap().closest_node(&src_coordinates), CH_GRAPH.as_ref().unwrap().closest_node(&tgt_coordinates)),
            _ => (GRAPH.as_ref().unwrap().closest_node(&src_coordinates), GRAPH.as_ref().unwrap().closest_node(&tgt_coordinates)),
        };
        // println!("Routing from {:?} to {:?}", src_node, tgt_node);

        let dijkstra_result = match ROUTING {
            Routing::DI => run_dijkstra(src_node, tgt_node, GRAPH.as_ref().unwrap()),
            Routing::BD => run_bidirectional_dijkstra(src_node, tgt_node, GRAPH.as_ref().unwrap(), true),
            Routing::ASTAR => run_a_star(src_node, tgt_node, GRAPH.as_ref().unwrap()),
            Routing::CH => run_ch(src_node, tgt_node, CH_GRAPH.as_ref().unwrap()),
        };
        
        match &dijkstra_result.path {
            Some(current_path) => {
                for i in 0..current_path.len() {
                    shortest_path.push([current_path[i].lat, current_path[i].lon]);
                }
            }
            None => ()
        }
        println!("Shortest path: {}", dijkstra_result);
        
    }


    shortest_path.into()
}

fn main() {
    let command = std::env::args_os().nth(1);
    let filename = param_to_string(2, Some("graph")).expect("Plese specify filename");

    match command {
        Some(command) => {
            match command.to_str() {
                Some("test_ch") => {
                    import_ch_graph(&filename);
                    test_random_samples_ch(unsafe { CH_GRAPH.as_ref().unwrap() });
                },
                Some("test_a*") => {
                    import_basic_graph(&filename);
                    test_random_samples_a_star(unsafe { GRAPH.as_ref().unwrap() });
                },
                Some("test_di") => {
                    import_basic_graph(&filename);
                    test_random_samples_dijkstra(unsafe { GRAPH.as_ref().unwrap() });
                },
                Some("test_bd") => {
                    import_basic_graph(&filename);
                    test_random_samples_bd(unsafe { GRAPH.as_ref().unwrap() });
                },
                Some("test_static") => {
                    import_basic_graph(&filename);
                    import_ch_graph(&filename);
                    test_samples(unsafe { GRAPH.as_ref().unwrap() }, unsafe { CH_GRAPH.as_ref().unwrap()})
                },
                Some("ch_precalc") => unsafe {
                    import_basic_graph(&filename);
                    let node_limit = match std::env::args_os().nth(3) {
                        Some(param) => {
                            param.into_string().expect("Something wrong with the node limit").parse::<u32>().expect("Node limit has wrong format")
                        },
                        None => 1,
                    };
                    new_ch_precalculations(GRAPH.as_ref().unwrap(), &("ch_".to_string() + &filename), node_limit);
                },
                Some("continue_ch_precalc") => {
                    let node_limit = match std::env::args_os().nth(3) {
                        Some(param) => {
                            param.into_string().expect("Something wrong with the node limit").parse::<u32>().expect("Node limit has wrong format")
                        },
                        None => 1,
                    };
                    continue_ch_precalculations(&("ch_".to_string() + &filename), node_limit)
                },
                Some("di") => unsafe {
                    ROUTING = Routing::DI;
                    import_basic_graph(&filename);
                    println!("Start Leaflet with Dijkstra routing");
                    run_tauri();
                },
                Some("bd") => unsafe {
                    ROUTING = Routing::BD;
                    import_basic_graph(&filename);
                    println!("Start Leaflet with bidirectional Dijkstra routing");
                    run_tauri();
                },
                Some("a*") => unsafe {
                    ROUTING = Routing::ASTAR;
                    import_basic_graph(&filename);
                    println!("Start Leaflet with A* routing");
                    run_tauri();
                },
                Some("ch") => unsafe {
                    ROUTING = Routing::CH;
                    import_ch_graph(&filename);
                    println!("Start Leaflet with contraction hierarchies routing");
                    run_tauri();
                },
                _ => println!("Command not known. Exit"),
            }
        },
        None => {
            import_ch_graph(&filename);
            println!("Default: Start Leaflet with contraction hierarchies routing");
            run_tauri();
        },
    }
}

fn import_basic_graph(filename: &str) {
    println!("Import Graph");
    unsafe {
        GRAPH = Some(import_graph_from_file(&filename).expect("Error importing Graph"));
    };
    println!("Finished importing");
}

fn import_ch_graph(filename: &str) {
    println!("Import CH Graph");
    unsafe {
        CH_GRAPH = Some(import_graph_from_file(&("ch_".to_string() + &filename)).expect("Error importing ch graph"));
    };
    println!("Finished importing");
}

fn run_tauri() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![route])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn param_to_string(nth: usize, alt_str: Option<&str>) -> Result<String, String> {
    match (std::env::args_os().nth(nth), alt_str) {
        (Some(osstring), _) => {
            let param = osstring.into_string().unwrap();
            return Ok(param)
        },
        (None, Some(alt_str)) => return Ok(String::from(alt_str)),
        (None, None) => return Err(format!("No {} parameter existing, but expected", nth))
    };
}