use std::time::SystemTime;
use crate::binary_minheap::BinaryMinHeap;
use graph_lib::{ShortestPathResult, Graph, Node};

/// Run a bidirectional Dijkstra from the source coodinates to the target coordinates
pub fn run_bidirectional_dijkstra(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {

    let now = SystemTime::now();
    
    let mut dijkstra_dists_forward = DijkstraDistances::init(graph.n_nodes(), src_node.id);
    let mut priority_queue_forward = BinaryMinHeap::with_capacity(graph.n_nodes());
    priority_queue_forward.push(src_node.id, &dijkstra_dists_forward.dists);

    let mut dijkstra_dists_backward = DijkstraDistances::init(graph.n_nodes(), tgt_node.id);
    let mut priority_queue_backward = BinaryMinHeap::with_capacity(graph.n_nodes());
    priority_queue_backward.push(tgt_node.id, &dijkstra_dists_backward.dists);

    let mut visited_nodes = 0;
    let mut node_id = 0 as usize;

    while !priority_queue_forward.is_empty() && !priority_queue_backward.is_empty() {
        if visited_nodes % 2 == 1 {
            node_id = priority_queue_forward.pop(&dijkstra_dists_forward.dists);
            if dijkstra_dists_backward.visited[node_id] {
                break;
            } else {
                process_edges(graph, node_id, &mut dijkstra_dists_forward, &mut priority_queue_forward);
            }
        } else {
            node_id = priority_queue_backward.pop(&dijkstra_dists_backward.dists);
            if dijkstra_dists_forward.visited[node_id] {
                break;
            } else {
                process_edges(graph, node_id, &mut dijkstra_dists_backward, &mut priority_queue_backward);
            }
        }
        visited_nodes += 1;
    }

    let mut path_forward = dijkstra_dists_forward.build_path(graph, node_id);
    let mut path_backward = dijkstra_dists_backward.build_path(graph, node_id);
    match (&mut path_forward, &mut path_backward) {
        (Some(path_forward), Some(path_backward)) => {
            path_forward.pop(); // otherwise node in the middle is double
            path_backward.reverse();
            path_forward.append(path_backward);
        },
        (_, _) => path_forward = None,
    }

    return ShortestPathResult {
        distance: dijkstra_dists_backward.dists[node_id] + dijkstra_dists_forward.dists[node_id],
        path: path_forward,
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes
    }
}

/// Process the outgoing edges of the node with id `node_id`
fn process_edges(graph: &Graph, node_id: usize, dijkstra_dists: &mut DijkstraDistances, pq: &mut BinaryMinHeap) {
    let node_dist = dijkstra_dists.dists[node_id];
    dijkstra_dists.visited[node_id] = true;
    for edge in graph.get_outgoing_edges(node_id) {
        let dist = node_dist + edge.dist;

        if dist < dijkstra_dists.dists[edge.tgt] {
            dijkstra_dists.dists[edge.tgt] = dist;
            dijkstra_dists.preds[edge.tgt] = node_id;

            pq.insert_or_update(edge.tgt, &dijkstra_dists.dists);
        }
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