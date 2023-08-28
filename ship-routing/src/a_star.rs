use std::time::SystemTime;
use crate::binary_minheap::BinaryMinHeap;
use graph_lib::{ShortestPathResult, Graph, Node};

/// Run a A* from the source coodinates to the target coordinates
pub fn run_a_star(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {

    let now = SystemTime::now();
    
    let mut heuristic_dists = HeuristicalDistances::init(graph.n_nodes(), src_node.id, src_node.distance_to_node(tgt_node).floor() as u32);

    let mut priority_queue = BinaryMinHeap::with_capacity(graph.n_nodes());
    priority_queue.push(src_node.id, &heuristic_dists.g_plus_h);

    let mut visited_nodes = 0;

    while !priority_queue.is_empty() {
        let node_id = priority_queue.pop(&heuristic_dists.g_plus_h);
        if node_id == tgt_node.id {
            break;
        } else {
            process_edges(graph, node_id, &mut heuristic_dists, &mut priority_queue, tgt_node);
        }
        visited_nodes += 1;
    }

    return ShortestPathResult {
        distance: heuristic_dists.g_plus_h[tgt_node.id],
        path: heuristic_dists.build_path(graph, tgt_node.id),
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes
    }
}

/// Process the outgoing edges of the node with id `node_id`
fn process_edges(graph: &Graph, node_id: usize, heuristic_dists: &mut HeuristicalDistances, pq: &mut BinaryMinHeap, tgt_node: &Node) {
    let central_node_dist = heuristic_dists.g_plus_h[node_id] - heuristic_dists.heuristic[node_id];
    for edge in graph.get_outgoing_edges(node_id) {
        if heuristic_dists.heuristic[edge.tgt] == u32::MAX {
            heuristic_dists.heuristic[edge.tgt] = graph.get_node(edge.tgt).distance_to_node(tgt_node).floor() as u32;
        }
        let neighbour_node_dist = central_node_dist + edge.dist + heuristic_dists.heuristic[edge.tgt];

        if neighbour_node_dist < heuristic_dists.g_plus_h[edge.tgt] {
            heuristic_dists.g_plus_h[edge.tgt] = neighbour_node_dist;
            heuristic_dists.preds[edge.tgt] = node_id;

            pq.insert_or_update(edge.tgt, &heuristic_dists.g_plus_h);
        }
    }
}


#[derive(Debug)]
pub struct HeuristicalDistances {
    g_plus_h: Vec<u32>,
    preds: Vec<usize>,
    heuristic: Vec<u32>
}

impl HeuristicalDistances {
    /// Creates a new `HeuristicalDistances` instance for given graph size with dist to src 0.0 and else infinity
    fn init(num_nodes: usize, src_id: usize, start_end_dist: u32) -> Self {
        let mut g_plus_h = vec![u32::MAX; num_nodes];
        let mut heuristic = vec![u32::MAX; num_nodes];
        heuristic[src_id] = start_end_dist;
        g_plus_h[src_id] = 0 + start_end_dist;
        Self {
            g_plus_h,
            preds: vec![usize::MAX; num_nodes],
            heuristic
        }
    }

    /// Build the path from the source node to the node with id `tgt_id`.
    /// This method assumes that the target can be reached from the source, otherwise it will
    /// output a path that solely consists of the target.
    pub fn build_path(&self, graph: &Graph, tgt_id: usize) -> Option<Vec<Node>> {
        if self.g_plus_h[tgt_id] == u32::MAX {return None}
        let mut path = vec![];
        let mut curr_pred = tgt_id;
        // source node has no predecessor
        while curr_pred < usize::MAX {
            path.push(graph.get_node(curr_pred).clone());
            curr_pred = self.preds[curr_pred];
        }
        path.reverse();
        return Some(path)
    }
}