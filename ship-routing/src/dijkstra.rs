use std::time::SystemTime;
use crate::binary_minheap::BinaryMinHeap;
use graph_lib::{ShortestPathResult, Graph, Node};

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_dijkstra(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {

    let now = SystemTime::now();
    
    let mut dijkstra_dists = DijkstraDistances::init(graph.n_nodes(), src_node.id);

    let mut pq = BinaryMinHeap::with_capacity(graph.n_nodes());
    pq.push(src_node.id, &dijkstra_dists.dists);

    let mut visited_nodes = 0;

    while !pq.is_empty() {
        let node_id = pq.pop(&dijkstra_dists.dists);
        if node_id == tgt_node.id {
            break;
        } else {
            process_edges(graph, node_id, &mut dijkstra_dists, &mut pq);
        }
        visited_nodes += 1;
    }

    return ShortestPathResult {
        distance: dijkstra_dists.dists[tgt_node.id],
        path: dijkstra_dists.build_path(graph, tgt_node.id),
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes
    }
}

/// Process the outgoing edges of the node with id `node_id`
fn process_edges(graph: &Graph, node_id: usize, result: &mut DijkstraDistances, pq: &mut BinaryMinHeap) {
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


#[derive(Debug)]
pub struct DijkstraDistances {
    dists: Vec<f32>,
    preds: Vec<usize>,
}

impl DijkstraDistances {
    /// Creates a new `DijkstraResult` instance for given graph size with dist to src 0.0 and else infinity
    fn init(num_nodes: usize, src_id: usize) -> Self {
        let mut dists = vec![f32::MAX; num_nodes];
        dists[src_id] = 0.0;
        Self {
            dists,
            preds: vec![usize::MAX; num_nodes],
        }
    }

    /// Build the path from the source node to the node with id `tgt_id`.
    /// This method assumes that the target can be reached from the source, otherwise it will
    /// output a path that solely consists of the target.
    pub fn build_path(&self, graph: &Graph, tgt_id: usize) -> Option<Vec<Node>> {
        if self.dists[tgt_id] == f32::MAX {return None}
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