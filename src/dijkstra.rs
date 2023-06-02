use std::error::Error;

use crate::datastructs::{Coordinates, ShortestPath, Graph};

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_dijkstra(src_coordinates: Coordinates, tgt_coordinates: Coordinates) -> Result<ShortestPath<'static>, Box<dyn Error>> {
    let graph = import_graph();

    let (src_node, tgt_node) = (graph.closest_node(src_coordinates), graph.closest_node(tgt_coordinates));
    
    Ok(ShortestPath::new(40000, vec![]))
}

fn import_graph() -> Graph {
    todo!()
}

