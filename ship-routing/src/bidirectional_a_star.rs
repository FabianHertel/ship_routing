use std::{time::SystemTime, collections::HashMap};
use crate::binary_minheap_map::BinaryMinHeapMap;
use graph_lib::{ShortestPathResult, Graph, Node};

/**
 * Run a A* from the source coodinates to the target coordinates
*/
pub fn run_bidirectional_a_star(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {
    let now = SystemTime::now();
    let start_end_dist = src_node.distance_to_node(src_node).floor() as u32;
    
    // init forward
    let mut heuristic_forward = HeuristicalDistances::init(src_node.id, start_end_dist);
    let mut pq_forward = BinaryMinHeapMap::with_capacity(graph.n_nodes());
    pq_forward.push(src_node.id, &heuristic_forward.g_plus_h);

    // init backward
    let mut heuristic_backward = HeuristicalDistances::init(tgt_node.id, start_end_dist);
    let mut pq_backward = BinaryMinHeapMap::with_capacity(graph.n_nodes());
    pq_backward.push(tgt_node.id, &heuristic_backward.g_plus_h);

    let mut visited_nodes = 0;
    let mut best_result_so_far = u32::MAX;
    let mut node_id_middle = None;
    let mut forward_finished = false;
    let mut backward_finished = false;

    println!("Time for initialization: {} sec", now.elapsed().unwrap().as_secs_f32());

    // iterate until in both directions nothing better can be found anymore
    while !forward_finished || !backward_finished {
        // try forward
        if !forward_finished && !pq_forward.is_empty() {
            let node_id_forward = pq_forward.pop(&heuristic_forward.g_plus_h);
            if heuristic_forward.g_plus_h[&node_id_forward] < best_result_so_far {
                if heuristic_backward.g.contains_key(&node_id_forward) {
                    let possible_result = heuristic_forward.g[&node_id_forward] + heuristic_backward.g[&node_id_forward];
                    if possible_result < best_result_so_far {
                        best_result_so_far = possible_result;
                        node_id_middle = Some(node_id_forward);
                        // println!("Best so far {}", best_result_so_far);
                    }
                }
                process_edges(graph, node_id_forward, &mut heuristic_forward, &mut pq_forward, tgt_node);
                visited_nodes += 1;
            } else {
                // println!("Forward finished, pq is {} and best is {}", heuristic_forward.g_plus_h[node_id_forward], best_result_so_far);
                forward_finished = true;
            }
        }
        // try backward
        if !backward_finished && !pq_backward.is_empty()  {
            let node_id_backward = pq_backward.pop(&heuristic_backward.g_plus_h);
            if heuristic_backward.g_plus_h[&node_id_backward] < best_result_so_far {
                if heuristic_forward.g.contains_key(&node_id_backward) {
                    let possible_result = heuristic_forward.g[&node_id_backward] + heuristic_backward.g[&node_id_backward];
                    if possible_result < best_result_so_far {
                        best_result_so_far = possible_result;
                        node_id_middle = Some(node_id_backward);
                        // println!("Best so far {}", best_result_so_far);
                    }
                }
                process_edges(graph, node_id_backward, &mut heuristic_backward, &mut pq_backward, tgt_node);
                visited_nodes += 1;
            } else {
                // println!("Backward finished, pq is {} and best is {}", heuristic_backward.g_plus_h[node_id_backward], best_result_so_far);
                backward_finished = true;
            }
        }
    }

    // build path
    let result_path = match node_id_middle {
        Some(node_id) => {
            let mut path_forward = heuristic_forward.build_path(graph, node_id).unwrap();
            let mut path_backward = heuristic_backward.build_path(graph, node_id).unwrap();
            path_forward.pop(); // otherwise node in the middle is double
            path_backward.reverse();
            path_forward.append(&mut path_backward);
            Some(path_forward)
        },
        None => None,
    };

    return ShortestPathResult {
        distance: best_result_so_far,
        path: result_path,
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes
    }
}

/**
 * visits a node, which means it processes all its edges and updates the function g(a)+h(a) of all neighbours
*/
fn process_edges(graph: &Graph, visiting_node_id: usize, heuristic_dists: &mut HeuristicalDistances, pq: &mut BinaryMinHeapMap, tgt_node: &Node) {
    let visiting_node_dist = heuristic_dists.g_plus_h[&visiting_node_id] - heuristic_dists.heuristic[&visiting_node_id];
    for edge in graph.get_outgoing_edges(visiting_node_id) {
        // if not calculated already, the heuristic is calculated here
        if !heuristic_dists.heuristic.contains_key(&edge.tgt) {
            heuristic_dists.heuristic.insert(edge.tgt,  graph.get_node(edge.tgt).distance_to_node(tgt_node).floor() as u32);
        }
        let neighbour_node_dist = visiting_node_dist + edge.dist + heuristic_dists.heuristic[&edge.tgt];
        
        // if way over visiting node is the best so far, the neighbour will be updated
        if !heuristic_dists.g_plus_h.contains_key(&edge.tgt) || neighbour_node_dist < heuristic_dists.g_plus_h[&edge.tgt] {
            heuristic_dists.g.insert(edge.tgt, visiting_node_dist + edge.dist);
            heuristic_dists.g_plus_h.insert(edge.tgt, neighbour_node_dist);
            heuristic_dists.preds.insert(edge.tgt, visiting_node_id);

            pq.insert_or_update(edge.tgt, &heuristic_dists.g_plus_h);
        }
    }
}



#[derive(Debug)]
pub struct HeuristicalDistances {
    g_plus_h: HashMap<usize, u32>,
    g: HashMap<usize, u32>,
    preds: HashMap<usize, usize>,
    heuristic: HashMap<usize, u32>
}

impl HeuristicalDistances {
    /// Creates a new `HeuristicalDistances` instance for given graph size with dist to src 0.0 and else infinity
    fn init(src_id: usize, start_end_dist: u32) -> Self {
        let mut g = HashMap::new();
        let mut g_plus_h = HashMap::new();
        let mut heuristic = HashMap::new();
        heuristic.insert(src_id, start_end_dist);
        g_plus_h.insert(src_id, 0 + start_end_dist);
        g.insert(src_id, 0);
        Self {
            g_plus_h,
            g,
            preds: HashMap::new(),
            heuristic
        }
    }

    /// Build the path from the source node to the node with id `tgt_id`.
    /// This method assumes that the target can be reached from the source, otherwise it will
    /// output a path that solely consists of the target.
    pub fn build_path(&self, graph: &Graph, tgt_id: usize) -> Option<Vec<Node>> {
        if !self.preds.contains_key(&tgt_id) {return None}
        let mut path = vec![];
        let mut curr_pred = tgt_id;
        // source node has no predecessor
        while curr_pred < usize::MAX {
            path.push(graph.get_node(curr_pred).clone());
            curr_pred = *self.preds.get(&curr_pred).unwrap();
        }
        path.reverse();
        return Some(path)
    }
}