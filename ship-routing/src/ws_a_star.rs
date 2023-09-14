use std::collections::HashMap;

use crate::{binary_minheap_map::BinaryMinHeap, ch::CHGraph};
use graph_lib::Coordinates;

/// Run a A* from the source coodinates to the target coordinates
#[inline]
pub fn ws_a_star(src: usize, tgt: usize, ch_graph: &CHGraph, edge_sum: u32, btw_node: usize) -> bool {
    
    let tgt_coordinates = ch_graph.borrow_node(tgt).coordinades;
    let mut heuristic_dists = HeuristicalDistances::init( src, ch_graph.borrow_node(src).coordinades.distance_to(&tgt_coordinates).floor() as u32);

    let mut priority_queue = BinaryMinHeap::with_capacity(ch_graph.n_nodes());
    priority_queue.push(src, &heuristic_dists.g_plus_h);

    while !priority_queue.is_empty() {
        let node_id = priority_queue.pop(&heuristic_dists.g_plus_h);
        if node_id == tgt || heuristic_dists.g_plus_h[&node_id] > edge_sum {
            break;
        } else {
            process_edges(ch_graph, node_id, &mut heuristic_dists, &mut priority_queue, &tgt_coordinates, btw_node);
        }
    }

    return !heuristic_dists.g_plus_h.contains_key(&tgt) || heuristic_dists.g_plus_h[&tgt] > edge_sum;
}

/// Process the outgoing edges of the node with id `node_id`
#[inline]
fn process_edges(ch_graph: &CHGraph, node_id: usize, heuristic_dists: &mut HeuristicalDistances, pq: &mut BinaryMinHeap, tgt_coordinates: &Coordinates, btw_node: usize) {
    let central_node_dist = heuristic_dists.g_plus_h[&node_id] - heuristic_dists.heuristic[&node_id];
    for (edge_tgt, edge) in &ch_graph.borrow_node(node_id).neighbours {
        if *edge_tgt != btw_node {
            if !heuristic_dists.heuristic.contains_key(edge_tgt) {
                heuristic_dists.heuristic.insert(*edge_tgt, ch_graph.borrow_node(*edge_tgt).coordinades.distance_to(tgt_coordinates).floor() as u32);
            }
            let neighbour_node_dist = central_node_dist + edge.dist + heuristic_dists.heuristic[edge_tgt];
    
            if !heuristic_dists.g_plus_h.contains_key(edge_tgt) || neighbour_node_dist < heuristic_dists.g_plus_h[edge_tgt] {
                heuristic_dists.g_plus_h.insert(*edge_tgt, neighbour_node_dist);
                heuristic_dists.preds.insert(*edge_tgt, node_id);
    
                pq.insert_or_update(*edge_tgt, &heuristic_dists.g_plus_h);
            }
        }
    }
}


#[derive(Debug)]
pub struct HeuristicalDistances {
    g_plus_h: HashMap<usize, u32>,
    preds: HashMap<usize, usize>,
    heuristic: HashMap<usize, u32>
}

impl HeuristicalDistances {
    /// Creates a new `HeuristicalDistances` instance for given graph size with dist to src 0.0 and else infinity
    pub fn init(src_id: usize, start_end_dist: u32) -> Self {
        let mut heuristic: HashMap<usize, u32> = HashMap::new();
        let mut g_plus_h: HashMap<usize, u32> = HashMap::new();
        heuristic.insert(src_id, start_end_dist);
        g_plus_h.insert(src_id, 0 + start_end_dist);
        Self {
            g_plus_h,
            preds: HashMap::new(),
            heuristic
        }
    }
}