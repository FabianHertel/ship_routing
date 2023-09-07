// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod dijkstra;
mod a_star;
mod bidirectional_dijkstra;
mod binary_minheap;
mod ch;
mod test_routing;
mod witness_search;

use graph_lib::{import_graph_from_file, Coordinates, Graph};
use test_routing::test_samples;

use crate::{bidirectional_dijkstra::run_bidirectional_dijkstra, ch::{ch_precalculations, run_ch}};

static mut GRAPH: Graph = Graph {
    nodes: Vec::new(),
    edges: Vec::new(),
    offsets: Vec::new(),
};
static mut CH_GRAPH: Graph = Graph {
    nodes: Vec::new(),
    edges: Vec::new(),
    offsets: Vec::new(),
};

#[tauri::command]
fn route(coordinates: [[f32;2];2]) -> Vec<[f32;2]> {

    let src_coordinates = Coordinates(coordinates[0][1], coordinates[0][0]);
    let tgt_coordinates = Coordinates(coordinates[1][1], coordinates[1][0]);
    let mut shortest_path = Vec::new();
    
    unsafe {
        let (src_node, tgt_node) = (GRAPH.closest_node(&src_coordinates), GRAPH.closest_node(&tgt_coordinates));
        // println!("Start dijkstra with start: {:?}, end: {:?}", src_node, tgt_node);
    
        let ch_result = run_ch(src_node, tgt_node, &CH_GRAPH);
        let dijkstra_result = run_bidirectional_dijkstra(src_node, tgt_node, &GRAPH);

        match &dijkstra_result.path {
            Some(current_path) => {
                for i in 0..current_path.len() {
                    shortest_path.push([current_path[i].lat, current_path[i].lon]);
                }
            }
            None => ()
        }

        match &ch_result.path {
            Some(current_path) => {
                for i in 0..current_path.len() {
                    shortest_path.push([current_path[i].lat, current_path[i].lon]);
                }
            }
            None => ()
        }
        println!("Dijkstra: {}", dijkstra_result);
        println!("CH: {}", ch_result);
    }


    shortest_path.into()
}

fn main() {
    let command = std::env::args_os().nth(1);
    println!("Import Graph");
    unsafe {
        GRAPH = import_graph_from_file("./data/graph.fmi").expect("Error importing Graph");
        CH_GRAPH = import_graph_from_file("./data/ch_graph.fmi").expect("Error importing Graph");
        // GRAPH.edges_to_clipboard();
    };
    println!("Finished importing");

    match command {
        Some(command) => {
            match command.to_str() {
                Some("test") => test_samples(unsafe { &GRAPH }, unsafe { &CH_GRAPH}),
                Some("ch_precalc") => unsafe {ch_precalculations(&GRAPH)},
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
