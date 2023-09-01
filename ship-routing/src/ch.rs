use std::{time::SystemTime, collections::{HashSet, HashMap}, cell::{RefCell, Ref}, borrow::BorrowMut};
use graph_lib::{ShortestPathResult, Graph, Node};
use cli_clipboard;

use crate::{binary_minheap::BinaryMinHeap, witness_search::run_witness_search};

pub fn ch_precalculations(graph: &Graph) {
    let mut contracting_graph = CHGraph::from_graph(graph);
    let mut l_counter: Vec<usize> = vec![0; contracting_graph.n_nodes()];
    let mut hopcount_sums: Vec<usize> = graph.nodes.iter().map(|node| graph.get_outgoing_edges(node.id).len()).collect();
    let mut hopcount_edges: Vec<usize> = vec![1; graph.n_edges()];
    let mut priority_queue = BinaryMinHeap::with_capacity(contracting_graph.n_nodes());

    let mut importance: Vec<f32> = vec![0.0; contracting_graph.n_nodes()];

    // initialize heuristic
    let mut insertions: Vec<Insertions> = vec![];
    for (node_id, node) in &contracting_graph.nodes {       // TODO: parallel
        let edges = &node.borrow().neighbours;
        let mut insert_edges: Vec<CHEdge> = vec![];
        let mut hopcount_sum = 0;

        for i in 0..edges.len() {
            for j in (i+1)..edges.len() {
                let edge_sum =  edges[i].dist + edges[j].dist;
                let src_node = edges[i].tgt;
                let tgt_node = edges[j].tgt;
                let witness_search = run_witness_search(
                    src_node, tgt_node, &contracting_graph, Some(50usize), edge_sum, node.borrow().id);
                if witness_search > edge_sum {
                    let hopcount = edges[i].hopcount + edges[j].hopcount;
                    insert_edges.push(CHEdge {dist: edge_sum, tgt: edges[i].tgt, hopcount});
                    hopcount_sum += hopcount;
                }
            }
        }
        importance[node.borrow().id] = l_counter[node.borrow().id] as f32 + insert_edges.len() as f32 / edges.len() as f32 + hopcount_sum as f32 / hopcount_sums[node.borrow().id] as f32;
        insertions.push(Insertions { hopcount_sum, insert_edges });

        priority_queue.push(node.borrow().id, &importance);
    }
    println!("Initial importance calculated, start contraction");

    // find independent subset
    let mut contracted_nodes: Vec<usize> = vec![];
    let mut neighbour_nodes: HashSet<usize> = HashSet::new();
    while !priority_queue.is_empty() {
        let next_node = priority_queue.pop(&importance);
        if !neighbour_nodes.contains(&next_node) {
            contracted_nodes.push(next_node);
            neighbour_nodes.extend(contracting_graph.borrow_node(next_node).neighbours.iter().map(|edge| edge.tgt));
        } else {
            // optional: without break not optimal, but faster
            break;
        }
    }
    let node_string = format!("[{}]", contracted_nodes.iter().map(|node_id| format!("[{},{}]", graph.get_node(*node_id).lon, graph.get_node(*node_id).lat)).reduce(|e,f| e+","+&f).unwrap());
    cli_clipboard::set_contents(node_string.to_owned()).unwrap();

    let first_contraction = contracted_nodes[0];
    println!("First out of {} nodes, node {}, with {} neigbours and {} inserted edges, removed hopcounts: {} and inserted hopcounts: {}",
        contracted_nodes.len(), first_contraction, contracting_graph.borrow_node(first_contraction).neighbours.len(),
        insertions[first_contraction].insert_edges.len(), hopcount_sums[first_contraction], insertions[first_contraction].hopcount_sum
    );

}

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_ch(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {
    let now = SystemTime::now();
    
    
    return ShortestPathResult {
        distance: u32::MAX,
        path: None,
        calculation_time: now.elapsed().unwrap().as_millis(),
        visited_nodes: 0
    }
}

pub struct CHGraph {
    nodes: HashMap<usize, RefCell<CHNode>>
}

impl CHGraph {

    pub fn new() -> CHGraph {
        return CHGraph {nodes: HashMap::new()};
    }

    pub fn from_graph(graph: &Graph) -> CHGraph {
        let mut new_graph = CHGraph::new();
        for node in &graph.nodes {
            new_graph.nodes.insert(node.id, RefCell::new(CHNode { 
                neighbours: vec![],
                id: node.id,
                // l: 0,
                // houcount_sum: graph.get_outgoing_edges(node.id).len(),
                // insertions: 
            }));
        }
        for edge in &graph.edges {
            let src_node = new_graph.nodes.get(&edge.src).unwrap();
            src_node.borrow_mut().neighbours.push(CHEdge { tgt: edge.tgt, hopcount: 1, dist: edge.dist });
        }
        return new_graph;
    }

    pub fn n_nodes(&self) -> usize {
        return self.nodes.len();
    }

    pub fn borrow_node(&self, id: usize) -> Ref<CHNode> {
        return self.nodes.get(&id).unwrap().borrow();
    }

}

pub struct CHNode {
    id: usize,
    pub neighbours: Vec<CHEdge>,
    // l: usize,
    // houcount_sum: usize,
    // insertions: Vec<Insertions>
}

struct Insertions {
    hopcount_sum: usize,
    insert_edges: Vec<CHEdge>
}

pub struct CHEdge {pub tgt: usize, pub hopcount: usize, pub dist: u32}