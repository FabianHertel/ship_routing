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
use test_routing::test_samples;

use crate::{bidirectional_dijkstra::run_bidirectional_dijkstra, ch::{ch_precalculations, run_ch}};

static mut GRAPH: Graph = Graph {
    nodes: Vec::new(),
    edges: Vec::new(),
    offsets: Vec::new(),
};
static mut CH_GRAPH: Option<Graph> = Some(Graph {
    nodes: Vec::new(),
    edges: Vec::new(),
    offsets: Vec::new(),
});

#[tauri::command]
fn route(coordinates: [[f32;2];2]) -> Vec<[f32;2]> {

    let src_coordinates = Coordinates(coordinates[0][1], coordinates[0][0]);
    let tgt_coordinates = Coordinates(coordinates[1][1], coordinates[1][0]);
    let mut shortest_path = Vec::new();
    
    unsafe {
        let (src_node, tgt_node) = (GRAPH.closest_node(&src_coordinates), GRAPH.closest_node(&tgt_coordinates));
        // println!("Routing from {:?} to {:?}", src_node, tgt_node);
    
        let dijkstra_result = run_bidirectional_dijkstra(src_node, tgt_node, &GRAPH, true);
        
        match &dijkstra_result.path {
            Some(current_path) => {
                for i in 0..current_path.len() {
                    shortest_path.push([current_path[i].lat, current_path[i].lon]);
                }
            }
            None => ()
        }
        println!("Dijkstra: {}", dijkstra_result);
        
        if CH_GRAPH.is_some() {
            let ch_result = run_ch(src_node, tgt_node, CH_GRAPH.as_ref().unwrap());
            match &ch_result.path {
                Some(current_path) => {
                    for i in 0..current_path.len() {
                        shortest_path.push([current_path[i].lat, current_path[i].lon]);
                    }
                }
                None => ()
            }
            println!("CH: {}", ch_result);
        } else {
            println!("No CH data found, so no CH calculation");
        }
    }


    shortest_path.into()
}

fn main() {
    let command = std::env::args_os().nth(1);
    let filename = "graph";
    println!("Import Graph");
    unsafe {
        GRAPH = import_graph_from_file(filename).expect("Error importing Graph");
        CH_GRAPH = import_graph_from_file(&("ch_".to_string() + &filename)).ok();
        // GRAPH.edges_to_clipboard();
    };
    println!("Finished importing");

    match command {
        Some(command) => {
            match command.to_str() {
                Some("test") => test_samples(unsafe { &GRAPH }, unsafe { CH_GRAPH.as_ref().unwrap()}),
                Some("ch_precalc") => unsafe {
                    let filename_out = param_to_string(2, Some("ch_graph")).expect("Plese specify filename");
                    ch_precalculations(&GRAPH, filename_out.as_str());
                },
                _ => println!("Command not known. Exit"),
            }
        },
        None => {
            tauri::Builder::default()
                .invoke_handler(tauri::generate_handler![route])
                .run(tauri::generate_context!())
                .expect("error while running tauri application");
        },
    }
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