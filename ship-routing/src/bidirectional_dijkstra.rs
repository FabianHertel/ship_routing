use std::time::SystemTime;
use crate::binary_minheap::BinaryMinHeap;
use graph_lib::{ShortestPathResult, Graph, Node};

/// Run a bidirectional Dijkstra from the source coodinates to the target coordinates
pub fn run_bidirectional_dijkstra(src_node: &Node, tgt_node: &Node, graph: &Graph, symmetric: bool) -> ShortestPathResult {

    let now = SystemTime::now();
    
    let mut dijkstra_forward = DijkstraDistances::init(graph.n_nodes(), src_node.id);
    let mut priority_queue_forward = BinaryMinHeap::with_capacity(graph.n_nodes());
    priority_queue_forward.push(src_node.id, &dijkstra_forward.dists);

    let mut dijkstra_backward = DijkstraDistances::init(graph.n_nodes(), tgt_node.id);
    let mut priority_queue_backward = BinaryMinHeap::with_capacity(graph.n_nodes());
    priority_queue_backward.push(tgt_node.id, &dijkstra_backward.dists);

    let mut visited_nodes = 0u32;
    let mut result_dist = u32::MAX;
    let mut node_id_middle = None;

    while !priority_queue_forward.is_empty() && !priority_queue_backward.is_empty() {
        let node_id_forward = priority_queue_forward.pop(&dijkstra_forward.dists);
        let node_id_backward = priority_queue_backward.pop(&dijkstra_backward.dists);
        dijkstra_forward.visited[node_id_forward] = true;
        dijkstra_backward.visited[node_id_backward] = true;
        let forward_dist = dijkstra_forward.dists[node_id_forward];
        let backward_dist = dijkstra_backward.dists[node_id_backward];

        for edge in graph.get_outgoing_edges(node_id_forward) {
            let edge_tgt_dist = forward_dist + edge.dist;
            if edge_tgt_dist < dijkstra_forward.dists[edge.tgt] {
                dijkstra_forward.dists[edge.tgt] = edge_tgt_dist;
                dijkstra_forward.preds[edge.tgt] = node_id_forward;
                priority_queue_forward.insert_or_update(edge.tgt, &dijkstra_forward.dists);
            }
            
            if dijkstra_backward.dists[edge.tgt] != u32::MAX && edge_tgt_dist + dijkstra_backward.dists[edge.tgt] < result_dist {
                result_dist = edge_tgt_dist + dijkstra_backward.dists[edge.tgt];
                node_id_middle = Some(edge.tgt);
            }

            // println!("forward: {}, {}, {}, {}, {}", edge.src, forward_dist, edge.tgt, edge.dist, result_dist);
        }

        for edge in graph.get_outgoing_edges(node_id_backward) {
            let edge_tgt_dist = backward_dist + edge.dist;
            if edge_tgt_dist < dijkstra_backward.dists[edge.tgt] {
                dijkstra_backward.dists[edge.tgt] = edge_tgt_dist;
                dijkstra_backward.preds[edge.tgt] = node_id_backward;
                priority_queue_backward.insert_or_update(edge.tgt, &dijkstra_backward.dists);
            }
            
            if dijkstra_forward.dists[edge.tgt] != u32::MAX && edge_tgt_dist + dijkstra_forward.dists[edge.tgt] < result_dist {
                result_dist = edge_tgt_dist + dijkstra_forward.dists[edge.tgt];
                node_id_middle = Some(edge.tgt);
            }
            // println!("backward: {}, {}, {}, {}, {}", edge.src, backward_dist, edge.tgt, edge.dist, result_dist);
        }

        visited_nodes += 2;
        if symmetric {
            if forward_dist + backward_dist >= result_dist {
                break;
            }
        } else {
            if forward_dist >= result_dist && backward_dist >= result_dist {
                break;
            }
        }
    }

    let result_path = match node_id_middle {
    Some(node_id) => {
            let mut path_forward = dijkstra_forward.build_path(graph, node_id).unwrap();
            let mut path_backward = dijkstra_backward.build_path(graph, node_id).unwrap();
            path_forward.pop(); // otherwise node in the middle is double
            path_backward.reverse();
            path_forward.append(&mut path_backward);
            Some(path_forward)
        },
        None => None,
    };

    return ShortestPathResult {
        distance: result_dist,
        path: result_path,
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes
    }
}

#[derive(Debug)]
pub struct DijkstraDistances {
    dists: Vec<u32>,
    preds: Vec<usize>,
    visited: Vec<bool>,
}

impl DijkstraDistances {
    /// Creates a new `DijkstraResult` instance for given graph size with dist to src 0.0 and else infinity
    fn init(num_nodes: usize, src_id: usize) -> Self {
        let mut dists = vec![u32::MAX; num_nodes];
        dists[src_id] = 0;
        Self {
            dists,
            preds: vec![usize::MAX; num_nodes],
            visited: vec![false; num_nodes],
        }
    }

    /// Build the path from the source node to the node with id `tgt_id`.
    /// This method assumes that the target can be reached from the source, otherwise it will
    /// output a path that solely consists of the target.
    pub fn build_path(&self, graph: &Graph, tgt_id: usize) -> Option<Vec<Node>> {
        if self.dists[tgt_id] == u32::MAX {return None}
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