use std::time::SystemTime;
use crate::binary_minheap::BinaryMinHeap;
use graph_lib::{ShortestPathResult, Graph, Node};

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_dijkstra(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {

    let now = SystemTime::now();
    
    let mut dijkstra_dists = DijkstraDistances::init(graph.n_nodes(), src_node.id);

    let mut priority_queue = BinaryMinHeap::with_capacity(graph.n_nodes());
    priority_queue.push(src_node.id, &dijkstra_dists.dists);

    let mut visited_nodes = 0;

    while !priority_queue.is_empty() {
        let node_id = priority_queue.pop(&dijkstra_dists.dists);
        if node_id == tgt_node.id {
            break;
        } else {
            process_edges(graph, node_id, &mut dijkstra_dists, &mut priority_queue);
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
fn process_edges(graph: &Graph, node_id: usize, dijkstra_dists: &mut DijkstraDistances, pq: &mut BinaryMinHeap) {
    let node_dist = dijkstra_dists.dists[node_id];
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
}

impl DijkstraDistances {
    /// Creates a new `DijkstraDistances` instance for given graph size with dist to src 0.0 and else infinity
    fn init(num_nodes: usize, src_id: usize) -> Self {
        let mut dists = vec![u32::MAX; num_nodes];
        dists[src_id] = 0;
        Self {
            dists,
            preds: vec![usize::MAX; num_nodes],
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