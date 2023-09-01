use crate::{binary_minheap::BinaryMinHeap, ch::CHGraph};

/// Run a bidirectional Dijkstra from the source coodinates to the target coordinates
pub fn run_witness_search(src_node: usize, tgt_node: usize, graph: &CHGraph, node_limit: Option<usize>, dist_limit: u32, ignore_node: usize, max_id: usize) -> u32 {
    
    let mut dijkstra_forward = DijkstraDistances::init(max_id, src_node);
    let mut priority_queue_forward = BinaryMinHeap::with_capacity(max_id);
    priority_queue_forward.push(src_node, &dijkstra_forward.dists);

    let mut dijkstra_backward = DijkstraDistances::init(max_id, tgt_node);
    let mut priority_queue_backward = BinaryMinHeap::with_capacity(max_id);
    priority_queue_backward.push(tgt_node, &dijkstra_backward.dists);

    let mut visited_nodes = 0u32;
    let mut result_dist: u32 = u32::MAX;

    while !priority_queue_forward.is_empty() && !priority_queue_backward.is_empty() {
        let node_id_forward = priority_queue_forward.pop(&dijkstra_forward.dists);
        let node_id_backward = priority_queue_backward.pop(&dijkstra_backward.dists);
        dijkstra_forward.visited[node_id_forward] = true;
        dijkstra_backward.visited[node_id_backward] = true;
        let forward_dist = dijkstra_forward.dists[node_id_forward];
        let backward_dist = dijkstra_backward.dists[node_id_backward];

        if forward_dist > dist_limit || backward_dist > dist_limit {
            break;
        } 

        for (tgt, edge) in &graph.borrow_node(node_id_forward).neighbours {
            let edge_tgt_dist = forward_dist + edge.dist;
            if *tgt != ignore_node {
                if edge_tgt_dist < dijkstra_forward.dists[*tgt] {
                    dijkstra_forward.dists[*tgt] = edge_tgt_dist;
                    dijkstra_forward.preds[*tgt] = node_id_forward;
                    priority_queue_forward.insert_or_update(*tgt, &dijkstra_forward.dists);
                }
                
                if dijkstra_backward.visited[*tgt] && edge_tgt_dist + dijkstra_backward.dists[*tgt] < result_dist {
                    result_dist = edge_tgt_dist + dijkstra_backward.dists[*tgt];
                }
            }
        }
        for (tgt, edge) in &graph.borrow_node(node_id_backward).neighbours {
            let edge_tgt_dist = backward_dist + edge.dist;
            if *tgt != ignore_node {
                if edge_tgt_dist < dijkstra_backward.dists[*tgt] {
                    dijkstra_backward.dists[*tgt] = edge_tgt_dist;
                    dijkstra_backward.preds[*tgt] = node_id_backward;
                    priority_queue_backward.insert_or_update(*tgt, &dijkstra_backward.dists);
                }
                
                if dijkstra_forward.visited[*tgt] && edge_tgt_dist + dijkstra_forward.dists[*tgt] < result_dist {
                    result_dist = edge_tgt_dist + dijkstra_forward.dists[*tgt];
                }
            }
        }

        visited_nodes += 2;
        if forward_dist + backward_dist >= result_dist {
            break;
        }
        if node_limit.is_some() && visited_nodes as usize >= node_limit.unwrap() {
            break;
        }
    }

    return result_dist
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
}