use std::time::SystemTime;
use graph_lib::{ShortestPathResult, Graph, Node};

use crate::{bidirectional_dijkstra::run_bidirectional_dijkstra_with_limit, binary_minheap::BinaryMinHeap};

pub fn ch_precalculations(graph: &Graph) {
    let mut l_counter: Vec<usize> = vec![0; graph.n_nodes()];
    let mut hopcount_sums: Vec<usize> = graph.nodes.iter().map(|node| graph.get_outgoing_edges(node.id).len()).collect();
    let mut hopcount_edges: Vec<usize> = vec![1; graph.n_edges()];
    let mut priority_queue = BinaryMinHeap::with_capacity(graph.n_nodes());

    let mut importance: Vec<f32> = vec![0.0; graph.n_nodes()];

    let mut insertions: Vec<Insertions> = vec![];
    for node in &graph.nodes {
        let edges = graph.get_outgoing_edges(node.id);
        let mut insert_edges: Vec<InsertionEdge> = vec![];
        let mut hopcount_sum = 0;

        for i in 0..edges.len() {
            for j in (i+1)..edges.len() {
                let edge_sum =  edges[i].dist + edges[j].dist;
                let src_node = graph.get_node(edges[i].tgt);
                let tgt_node = graph.get_node(edges[j].tgt);
                let witness_search = run_bidirectional_dijkstra_with_limit(
                    src_node, tgt_node, graph, Some(50usize), Some(edge_sum), Some(node.id));
                if witness_search.path.is_none() || witness_search.distance > edge_sum {
                    let node_offset = graph.offsets[node.id];
                    let hopcount = hopcount_edges[node_offset + i] + hopcount_edges[node_offset + j];
                    insert_edges.push(InsertionEdge {dist: edge_sum, tgt: edges[i].tgt, hopcount});
                    hopcount_sum += hopcount;
                }
            }
        }
        importance[node.id] = (l_counter[node.id] as f32 + insert_edges.len() as f32 / edges.len() as f32 + hopcount_sum as f32 / hopcount_sums[node.id] as f32);
        insertions.push(Insertions { hopcount_sum, insert_edges });

        priority_queue.push(node.id, &importance);
    }
    println!("Initial importance calculated, start contraction");

    let first_contraction = priority_queue.pop(&importance);
    println!("{:?}", importance);
    println!("node {}, with {} neigbours and {} inserted edges, removed hopcounts: {} and inserted hopcounts: {}",
        first_contraction, graph.get_outgoing_edges(first_contraction).len(), insertions[first_contraction].insert_edges.len(),
        hopcount_sums[first_contraction], insertions[first_contraction].hopcount_sum
    )
}

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_ch(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {
    let now = SystemTime::now();
    
    
    return ShortestPathResult {
        distance: u32::MAX,
        path: None,
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes: 0
    }
}

struct Insertions {
    hopcount_sum: usize,
    insert_edges: Vec<InsertionEdge>
}

struct InsertionEdge {tgt: usize, hopcount: usize, dist: u32}