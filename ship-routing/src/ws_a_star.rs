use std::{collections::HashMap, cell::RefCell};

use crate::{binary_minheap_map::BinaryMinHeapMap, ch::CHGraph};
use graph_lib::Coordinates;

/// Run a A* from the source coodinates to the target coordinates
#[inline]
pub fn ws_a_star(src: usize, tgt: usize, ch_graph: &CHGraph, edge_sum: u32, btw_node: usize, a_star_object: &AStartObject) -> bool {
    
    let tgt_coordinates = ch_graph.borrow_node(tgt).coordinades;

    a_star_object.0.borrow_mut().clear(src, tgt, ch_graph.borrow_node(src).coordinades.distance_to(&tgt_coordinates).floor() as u32);
    a_star_object.1.borrow_mut().clear();
    a_star_object.1.borrow_mut().push(src, &a_star_object.0.borrow().g_plus_h);

    while !a_star_object.1.borrow().is_empty() {
        let node_id = a_star_object.1.borrow_mut().pop(&a_star_object.0.borrow().g_plus_h);
        if node_id == tgt || a_star_object.0.borrow().g_plus_h[&node_id] > edge_sum || a_star_object.0.borrow().g_plus_h[&tgt] < edge_sum {
            break;
        } else {
            process_edges(ch_graph, node_id, a_star_object, &tgt_coordinates, btw_node);
        }
    }

    return a_star_object.0.borrow().g_plus_h[&tgt] > edge_sum;
}

/// Process the outgoing edges of the node with id `node_id`
#[inline]
fn process_edges(ch_graph: &CHGraph, node_id: usize, a_star_object: &AStartObject, tgt_coordinates: &Coordinates, btw_node: usize) {
    let central_node_dist = a_star_object.0.borrow().g_plus_h[&node_id] - a_star_object.0.borrow().heuristic[&node_id];
    for (edge_tgt, edge) in &ch_graph.borrow_node(node_id).neighbours {
        if *edge_tgt != btw_node {
            if !a_star_object.0.borrow().heuristic.contains_key(edge_tgt) {
                a_star_object.0.borrow_mut().heuristic.insert(*edge_tgt, ch_graph.borrow_node(*edge_tgt).coordinades.distance_to(tgt_coordinates).floor() as u32);
            }
            let neighbour_node_dist = central_node_dist + edge.dist + a_star_object.0.borrow().heuristic[edge_tgt];
    
            if !a_star_object.0.borrow().g_plus_h.contains_key(edge_tgt) || neighbour_node_dist < a_star_object.0.borrow().g_plus_h[edge_tgt] {
                a_star_object.0.borrow_mut().g_plus_h.insert(*edge_tgt, neighbour_node_dist);
                a_star_object.0.borrow_mut().preds.insert(*edge_tgt, node_id);
    
                a_star_object.1.borrow_mut().insert_or_update(*edge_tgt, &a_star_object.0.borrow().g_plus_h);
            }
        }
    }
}

pub type AStartObject = (RefCell<HeuristicalDistances>, RefCell<BinaryMinHeapMap>);


#[derive(Debug)]
pub struct HeuristicalDistances {
    g_plus_h: HashMap<usize, u32>,
    preds: HashMap<usize, usize>,
    heuristic: HashMap<usize, u32>
}

impl HeuristicalDistances {
    /// Creates a new `HeuristicalDistances` instance for given graph size with dist to src 0.0 and else infinity
    pub fn init() -> Self {
        Self {
            g_plus_h: HashMap::new(),
            preds: HashMap::new(),
            heuristic: HashMap::new()
        }
    }

    pub fn clear(&mut self, src_id: usize, tgt: usize, start_end_dist: u32) {
        self.heuristic.clear();
        self.g_plus_h.clear();
        self.preds.clear();
        self.heuristic.insert(src_id, start_end_dist);
        self.heuristic.insert(tgt, 0);
        self.g_plus_h.insert(src_id, 0 + start_end_dist);
        self.g_plus_h.insert(tgt, u32::MAX);
    }
}