use std::{error::Error};

use crate::{datastructs::{Coordinates, ShortestPath, Graph, Node}, binary_minheap_alex::BinaryMinHeap};

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_dijkstra(src_coordinates: Coordinates, tgt_coordinates: Coordinates) -> Result<Option<ShortestPath>, Box<dyn Error>> {
    let graph = &import_graph();

    let (src_node, tgt_node) = (graph.closest_node(src_coordinates), graph.closest_node(tgt_coordinates));

    let (mut result, mut pq) = init_result_and_pq(graph, src_node.id);

    while !pq.is_empty() {
        let node_id = pq.pop(&result.dists);
        if node_id == tgt_node.id {
            break;
        } else {
            process_edges(graph, node_id, &mut result, &mut pq);
        }
    }

    Ok(result.result_of(graph, tgt_node.id))
}

fn import_graph() -> Graph {
    todo!()
}


/// Initialize the `DijkstraResult` instance and the priority queue for a run of the Dijkstra algorithm
fn init_result_and_pq(graph: &Graph, src_id: usize) -> (DijkstraResult, BinaryMinHeap) {
    let mut result = DijkstraResult::new(graph.n_nodes());
    result.dists[src_id] = 0.0;

    let mut pq = BinaryMinHeap::with_capacity(graph.n_nodes());
    pq.push(src_id, &result.dists);

    (result, pq)
}

/// Process the outgoing edges of the node with id `node_id`
fn process_edges(graph: &Graph, node_id: usize, result: &mut DijkstraResult, pq: &mut BinaryMinHeap) {
    let node_dist = result.dists[node_id];
    for edge in graph.get_outgoing_edges(node_id) {
        let dist = node_dist + edge.dist;

        if dist < result.dists[edge.tgt] {
            result.dists[edge.tgt] = dist;
            result.preds[edge.tgt] = node_id;

            pq.insert_or_update(edge.tgt, &result.dists);
        }
    }
}

pub struct DijkstraResult {
    dists: Vec<f64>,
    preds: Vec<usize>,
}

impl DijkstraResult {
    /// Creates a new `DijkstraResult` instance for given graph size
    fn new(num_nodes: usize) -> Self {
        Self {
            dists: vec![f64::MAX; num_nodes],
            preds: vec![usize::MAX; num_nodes],
        }
    }

    /// Returns the dijkstra result for the node with id `node_id` in a `Some` or `None` if the
    /// node is not reachable from the source node
    pub fn result_of<'a>(&self, graph: &'a Graph, node_id: usize) -> Option<ShortestPath> {
        if self.dists[node_id] == f64::MAX { None } else {
            Some(ShortestPath::new(self.dists[node_id], self.build_path(graph, node_id)))
        }
    }

    /// Build the path from the source node to the node with id `tgt_id`.
    /// This method assumes that the target can be reached from the source, otherwise it will
    /// output a path that solely consists of the target.
    pub fn build_path(&self, graph: &Graph, tgt_id: usize) -> Vec<Node> {
        let mut path = vec![];
        let mut curr_pred = tgt_id;
        // source node has no predecessor
        while curr_pred < usize::MAX {
            path.push(graph.get_node(curr_pred).clone());
            curr_pred = self.preds[curr_pred];
        }
        path.reverse();
        path
    }
}