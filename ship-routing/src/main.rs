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
fn route(coordinates: [[f64;2];2]) -> Vec<[f64;2]> {

    let src = Coordinates(coordinates[0][1], coordinates[0][0]);
    let tgt = Coordinates(coordinates[1][1], coordinates[1][0]);
    let mut shortest_path = Vec::new();

    unsafe{
        let path = run_dijkstra(src, tgt, &GRAPH).expect("Error Dijkstra");
        
        match path {
            Some(current_path) => {
                    for i in 0..current_path.path().len() {
                        shortest_path.push([current_path.path()[i].lat, current_path.path()[i].lon]);
                    }
            }
            None => println!("No Solution found")
        }
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
