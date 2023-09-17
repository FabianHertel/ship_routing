use std::collections::HashSet;

use graph_lib::{Coordinates, Graph, file_interface::print_graph_to_file};

/**
 * Extract black sea as a subgraph of the whole. Done by deep search of a point in the center of Black Sea, so only working if Black Sea is disconnected to the rest.
 * Used to display a part of the graph and to test routing methods on a smaller problem
 */
pub fn extract_black_sea(graph: &Graph) {
    let in_black_sea = Coordinates(31.80666484258255, 44.0467677172621);
    let node_in_black_sea = graph.closest_node(&in_black_sea);
    let mut nodes_to_check: Vec<usize> = graph.get_outgoing_edges(node_in_black_sea.id).to_vec().iter().map(|e| e.tgt).collect();
    let mut found_nodes: HashSet<usize> = nodes_to_check.clone().into_iter().collect();

    println!("Start deep search");
    while nodes_to_check.len() > 1 {
        let node = nodes_to_check.pop().unwrap();
        for edge in graph.get_outgoing_edges(node) {
            if !found_nodes.contains(&edge.tgt) {
                found_nodes.insert(edge.tgt);
                nodes_to_check.push(edge.tgt);
            }
        }
    }

    let mut black_sea = graph.subgraph(found_nodes.into_iter().collect());
    black_sea.edges_to_clipboard();
    println!("nodes: {}, edges: {}", black_sea.n_nodes(), black_sea.edges.len());
    print_graph_to_file(&black_sea.nodes, &mut black_sea.edges, "black_sea");
    println!("Graph exported in file")
}