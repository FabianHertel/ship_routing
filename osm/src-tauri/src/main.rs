// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod dijkstra;
mod datastructs;
mod binary_minheap;
mod graph;

use crate::datastructs::Coordinates;
use crate::graph::Graph;


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn route(coordinates: [[f64;2];2]) -> Vec<[f64;2]> {

    println!("{:?}",coordinates);

    let src = Coordinates(coordinates[0][0], coordinates[0][1]);
    let tgt = Coordinates(coordinates[1][0], coordinates[1][1]);

    // let graph_file_path = "./data/graph";
    // let graph_import: Graph = import_graph_from_file(graph_file_path).expect("Error importing Graph");

    //let mut path = run_dijkstra(src, tgt, &graph_import);
    let mut path = Vec::new();
    path.push(coordinates[0]);
    path.push(coordinates[1]);

    path.into()
}

fn main() {
    
    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![route])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

}
