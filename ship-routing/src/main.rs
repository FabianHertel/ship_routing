// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod dijkstra;
mod binary_minheap;

use graph_lib::{import_graph_from_file, Coordinates, Graph};
use crate::dijkstra::run_dijkstra;

static mut GRAPH: Graph = Graph {
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
    
        let dijkstra_result = run_dijkstra(src_node, tgt_node, &GRAPH);

        match &dijkstra_result.path {
            Some(current_path) => {
                for i in 0..current_path.len() {
                    shortest_path.push([current_path[i].lat, current_path[i].lon]);
                }
            }
            None => ()
        }
        println!("Finished dijkstra from {} to {} with {}", src_node.id, tgt_node.id, dijkstra_result);
    }


    shortest_path.into()
}

fn main() {
    tauri::Builder::default()
    .setup(|_app| {
        println!("Import Graph");
        unsafe{
            GRAPH = import_graph_from_file("./data/graph.fmi").expect("Error importing Graph");
        }
        println!("Finished importing");
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![route])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
