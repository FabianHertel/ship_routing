use std::{time::SystemTime, collections::{HashSet, HashMap}, cell::{RefCell, Ref, RefMut}};
use graph_lib::{ShortestPathResult, Graph, Node};
use cli_clipboard;
use crate::{binary_minheap::BinaryMinHeap, witness_search::run_witness_search};

pub fn ch_precalculations(graph: &Graph) {
    let mut contracting_graph = CHGraph::from_graph(graph);
    println!("Graph of size {} and {} edges", contracting_graph.n_nodes(), contracting_graph.n_edges());
    let mut l_counter: Vec<usize> = vec![0; contracting_graph.n_nodes()];
    let mut priority_queue: BinaryMinHeap = BinaryMinHeap::with_capacity(contracting_graph.n_nodes());
    let mut importance: Vec<f32> = vec![0.0; contracting_graph.n_nodes()];

    let mut update_nodes = contracting_graph.nodes.keys().map(|node| *node).collect();

    while contracting_graph.n_nodes() as f32 > graph.n_nodes() as f32 * 0.1 {
        contracting_graph.update_importance_of(&mut importance, &update_nodes, &mut priority_queue, &l_counter, graph.n_nodes());

        let (independent_set, affected_nodes) = contracting_graph.find_best_independent_set(&mut priority_queue, &importance);
        contracting_graph.nodes_to_clipboard(graph, &independent_set);
        update_nodes = affected_nodes;
        println!("found independent set");
        // std::io::stdin().read_line(&mut String::new());

        contracting_graph.contract_nodes(independent_set, &mut l_counter);
        println!("Contracted: Graph of size {} and {} edges", contracting_graph.n_nodes(), contracting_graph.n_edges());
        contracting_graph.edges_to_clipboard(graph);
    }
    
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
                neighbours: HashMap::new(),
                id: node.id,
                // l: 0,
                // houcount_sum: graph.get_outgoing_edges(node.id).len(),
                insertions: vec![]
            }));
        }
        for edge in &graph.edges {
            let src_node = new_graph.nodes.get(&edge.src).unwrap();
            src_node.borrow_mut().neighbours.insert(edge.tgt, CHEdge { hopcount: 1, dist: edge.dist });
        }
        return new_graph;
    }

    pub fn n_nodes(&self) -> usize {
        return self.nodes.len();
    }

    pub fn borrow_node(&self, id: usize) -> Ref<CHNode> {
        return self.nodes.get(&id).unwrap().borrow();
    }

    pub fn borrow_mut_node(&self, id: usize) -> RefMut<CHNode> {
        return self.nodes.get(&id).expect(&format!("No node with id {}", id)).borrow_mut();
    }

    pub fn update_importance_of(&self, importance: &mut Vec<f32>, update_nodes: &HashSet<usize>, priority_queue: &mut BinaryMinHeap, l_counter: &Vec<usize>, max_id: usize) {
        let now = SystemTime::now();
        for node_id in update_nodes {       // TODO: parallel
            let mut hopcount_sum_insert = 0;
            let mut hopcount_sum = 0f32;
            let mut insert_edges: Vec<(usize, usize, CHEdge)> = vec![];
            {
                let node = self.borrow_node(*node_id);
                let neighbours = &node.neighbours;
                let neighbour_ids: Vec<&usize> = neighbours.keys().collect();

                hopcount_sum += neighbours.iter().fold(0.0, |base, e| base + e.1.hopcount as f32);

                for i in 0..neighbour_ids.len() {
                    for j in (i+1)..neighbour_ids.len() {
                        let i_edge = neighbours.get(neighbour_ids[i]).unwrap();
                        let j_edge = neighbours.get(neighbour_ids[j]).unwrap();
                        let edge_sum =  i_edge.dist + j_edge.dist;
                        let src_node = neighbour_ids[i];
                        let tgt_node = neighbour_ids[j];
                        let witness_search = run_witness_search(
                            *src_node, *tgt_node, self, Some(50usize), edge_sum, *node_id, max_id);
                        if witness_search > edge_sum {
                            let hopcount = i_edge.hopcount + j_edge.hopcount;
                            insert_edges.push((*neighbour_ids[i], *neighbour_ids[j], CHEdge {dist: edge_sum, hopcount}));
                            hopcount_sum_insert += hopcount;
                        }
                    }
                }
                importance[node.id] = l_counter[node.id] as f32 + insert_edges.len() as f32 / neighbour_ids.len() as f32 + hopcount_sum_insert as f32 / hopcount_sum as f32;
                priority_queue.insert_or_update(node.id, &importance);
            } {
                self.borrow_mut_node(*node_id).insertions = insert_edges;
            }
        }
        println!("Updated importance of {} nodes in {} ms", update_nodes.len(), now.elapsed().unwrap().as_millis());
    }

    pub fn find_best_independent_set(&self, priority_queue: &mut BinaryMinHeap, importance: &Vec<f32>) -> (Vec<usize>, HashSet<usize>) {
        let mut independent_set: Vec<usize> = vec![];
        let mut neighbour_nodes: HashSet<usize> = HashSet::new();
        let mut importance_limit = f32::MAX;
        while !priority_queue.is_empty() {
            let next_node = priority_queue.pop(&importance);
            if importance[next_node] > importance_limit {
                // optional: without break not optimal, but faster
                break;
            }
            if !neighbour_nodes.contains(&next_node) {
                independent_set.push(next_node);
                neighbour_nodes.extend(self.borrow_node(next_node).neighbours.iter().map(|(tgt, _)| tgt));
            } else if importance_limit == f32::MAX {
                importance_limit = importance[next_node] + 1.0;
                println!("Importance limit: {}", importance_limit);
            }
        }
        return (independent_set, neighbour_nodes);
    }

    pub fn contract_nodes(&mut self, nodes: Vec<usize>, l_counter: &mut Vec<usize>) {
        nodes.iter().for_each(|node_id| {
            // println!("remove node {}", node_id);
            let removed_node = self.nodes.remove(node_id).expect(&format!("contraction node with id {} doesn't exist", node_id));

            // remove from neighbours and increment their l counter
            for (tgt, _) in &removed_node.borrow().neighbours {
                // println!("remove {} from {}", node_id, tgt);
                self.borrow_mut_node(*tgt).neighbours.remove(&node_id);
                l_counter[*tgt] += 1;
            }

            // add insertions
            for (src, tgt, new_edge) in &removed_node.borrow().insertions {
                // println!("insert edge from {} to {}", src, tgt);
                self.borrow_mut_node(*tgt).neighbours.insert(*src, new_edge.clone());
                self.borrow_mut_node(*src).neighbours.insert(*tgt, new_edge.to_owned());
            }
        });
    }
    
    pub fn n_edges(&self) -> usize {
        self.nodes.iter().fold(0usize, |base,element| base + element.1.borrow().neighbours.len())
    }

    pub fn edges_to_clipboard(&self, graph: &Graph) {
        let mut the_string = "[".to_string();
        the_string += &self.nodes.iter().map(|(src, node)| {
            node.borrow().neighbours.iter().map(|(tgt, _)| format!("[[{},{}],[{},{}]]", graph.nodes[*src].lon, graph.nodes[*src].lat, graph.nodes[*tgt].lon, graph.nodes[*tgt].lat)).reduce(|e,f| e + "," + &f).unwrap()
        }).reduce(|e,f| e + "," + &f).unwrap();
        the_string += "]";
        cli_clipboard::set_contents(the_string.to_owned()).unwrap();
        assert_eq!(cli_clipboard::get_contents().unwrap(), the_string);
    }

    pub fn nodes_to_clipboard(&self, graph: &Graph, nodes: &Vec<usize>) {
        let node_string = format!("[{}]", nodes.iter().map(|node_id| format!("[{},{}]", graph.get_node(*node_id).lon, graph.get_node(*node_id).lat)).reduce(|e,f| e+","+&f).unwrap());
        cli_clipboard::set_contents(node_string.to_owned()).unwrap();
    }
}

pub struct CHNode {
    id: usize,
    pub neighbours: HashMap<usize, CHEdge>,
    // l: usize,
    // houcount_sum: usize,
    insertions: Insertions
}

type Insertions = Vec<(usize, usize, CHEdge)>;

#[derive(Clone)]
pub struct CHEdge {pub hopcount: usize, pub dist: u32}