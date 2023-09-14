use std::{time::SystemTime, collections::{HashSet, HashMap}, cell::{RefCell, Ref, RefMut}, fs::File, io::{Write, self, BufReader}};
use graph_lib::{ShortestPathResult, Graph, Node, Edge, file_interface::print_graph_to_file, Coordinates};
use cli_clipboard;
use crate::{binary_minheap::BinaryMinHeap, ws_a_star::{ws_a_star, AStartObject, HeuristicalDistances}, bidirectional_dijkstra::run_bidirectional_dijkstra, binary_minheap_map::BinaryMinHeapMap};

pub fn new_ch_precalculations(graph: &Graph, filename_out: &str) {
    println!("Convert graphs");
    let contracting_graph = CHGraph::from_graph(&graph.nodes, &graph.edges);
    let final_ch_graph = CHGraph::from_graph(&graph.nodes, &graph.edges);       // will be the upwareded DAG

    let ws_object: Vec<HashMap<usize, u32>> = vec![HashMap::new(); graph.n_nodes()];

    println!("Graph of size {} and {} edges", contracting_graph.n_nodes(), contracting_graph.n_edges());
    let l_counter: Vec<usize> = vec![0; contracting_graph.n_nodes()];
    let priority_queue: BinaryMinHeap = BinaryMinHeap::with_capacity(contracting_graph.n_nodes());
    let importance: Vec<f32> = vec![0.0; contracting_graph.n_nodes()];
    let update_nodes = contracting_graph.nodes.keys().map(|node| *node).collect();
    let level_counter = 0;
    
    ch_precalculations(contracting_graph, final_ch_graph, ws_object, l_counter, priority_queue, importance, update_nodes, level_counter, filename_out);
}

pub fn continue_ch_precalculations(filename_out: &str) {
    println!("Reading temp file");
    let graph_file = File::open("data/graph/ch_temp.bin").expect("CH temp file to continue not found, abort");
    let reader = BufReader::new(graph_file);

    let decoded: Result<(Vec<Node>, Vec<Edge>, Vec<Node>, Vec<Edge>, Vec<f32>, Vec<usize>, Vec<HashMap<usize,u32>>, HashSet<usize>, i32), Box<bincode::ErrorKind>> = 
        bincode::deserialize_from(reader);
    match decoded {
        Ok((
            final_ch_graph_fmi_nodes, final_ch_graph_fmi_edges, contracted_graph_fmi_nodes,
            contracted_graph_fmi_edges, importance, l_counter, ws_object, 
            update_nodes, level_counter
        )) => {
            let contracted_graph = CHGraph::from_graph(&contracted_graph_fmi_nodes, &contracted_graph_fmi_edges);
            let final_ch_graph = CHGraph::from_graph(&final_ch_graph_fmi_nodes, &final_ch_graph_fmi_edges);
            let mut priority_queue: BinaryMinHeap = BinaryMinHeap::with_capacity(final_ch_graph.n_nodes());
            for node in contracted_graph_fmi_nodes {
                priority_queue.insert_or_update(node.id, &importance);
            }

            println!("Restored last ch generation session with graph size {} {}", contracted_graph.n_nodes(), contracted_graph.n_edges());

            ch_precalculations(contracted_graph, final_ch_graph, ws_object, l_counter, priority_queue, importance, update_nodes, level_counter, filename_out);
        },
        Err(_) => println!("CH temp file to continue not in the right format, abort"),
    }
    
}

pub fn ch_precalculations(
    mut contracting_graph: CHGraph, mut final_ch_graph: CHGraph, mut ws_object: Vec<HashMap<usize, u32>>, mut l_counter: Vec<usize>,
    mut priority_queue: BinaryMinHeap, mut importance: Vec<f32>, mut update_nodes: HashSet<usize>, mut level_counter: i32,
    filename_out: &str
) {
    let now = SystemTime::now();
    let mut last_save = SystemTime::now();
    let mut last_save_durance = 10;
    let a_star_object: AStartObject = (RefCell::new(HeuristicalDistances::init()), RefCell::new(BinaryMinHeapMap::with_capacity(contracting_graph.n_nodes())));
    while contracting_graph.n_nodes() as f32 > final_ch_graph.n_nodes() as f32 * 0.01 {
        contracting_graph.update_importance_of(&mut importance, &update_nodes, &mut priority_queue, &l_counter, &mut ws_object, &a_star_object);

        let (independent_set, affected_nodes) = contracting_graph.find_best_independent_set(&mut priority_queue, &importance);
        update_nodes = affected_nodes;
        // contracting_graph.nodes_and_edges_to_clipboard(graph, &independent_set);
        
        contracting_graph.contract_nodes(&independent_set, &mut l_counter, &mut final_ch_graph);
        level_counter += 1;

        if last_save.elapsed().unwrap().as_secs() > 10 * last_save_durance {
            print!("Save; ");
            let now = SystemTime::now();
            save_in_file(&final_ch_graph, &contracting_graph, &importance, &l_counter, &ws_object, &update_nodes, level_counter);
            last_save = SystemTime::now();
            last_save_durance = now.elapsed().unwrap().as_secs();
        }
        let percent = (contracting_graph.n_nodes() * 100) as f32 / final_ch_graph.n_nodes() as f32;
        println!("Contracted: graph of size {:.2}% - {} n - {} e; final {} edges",
            percent, contracting_graph.n_nodes(), contracting_graph.n_edges(), final_ch_graph.n_edges()
        );
    }
    
    println!("Finished graph contraction in {} min, final graph has {} levels, {} nodes on top level and {} edges",
        now.elapsed().unwrap().as_secs_f64() / 60.0, level_counter, contracting_graph.n_nodes(), final_ch_graph.n_edges());

    let mut final_fmi_graph = final_ch_graph.get_print_nodes_and_edges();
    print_graph_to_file(&final_fmi_graph.0, &mut final_fmi_graph.1, filename_out);
}

fn save_in_file(
    final_ch_graph: &CHGraph, contracting_graph: &CHGraph, importance: &Vec<f32>, l_counter: &Vec<usize>,
    ws_object: &Vec<HashMap<usize, u32>>, update_nodes: &HashSet<usize>, level_counter: i32
) {
    let final_ch_graph_fmi = final_ch_graph.get_print_nodes_and_edges();
    let contracted_graph_fmi = contracting_graph.get_print_nodes_and_edges();
    let encoded = bincode::serialize(&(
        final_ch_graph_fmi.0, final_ch_graph_fmi.1, contracted_graph_fmi.0, contracted_graph_fmi.1,
        importance, l_counter, ws_object, update_nodes, level_counter
    )).unwrap();
    
    let mut f = File::create("data/graph/ch_temp.bin").expect("Unable to create file");
    f.write_all(encoded.as_slice()).expect("unable to write file");
}

/// Run a Dijkstra from the source coodinates to the target coordinates
pub fn run_ch(src_node: &Node, tgt_node: &Node, graph: &Graph) -> ShortestPathResult {
    return run_bidirectional_dijkstra(src_node, tgt_node, graph, false);
}

pub struct CHGraph {
    nodes: HashMap<usize, RefCell<CHNode>>,
}

impl CHGraph {

    pub fn new() -> CHGraph {
        return CHGraph {nodes: HashMap::new()};
    }

    pub fn from_graph(nodes: &Vec<Node>, edges: &Vec<Edge>) -> CHGraph {
        let mut new_graph = CHGraph::new();
        for node in nodes {
            new_graph.nodes.insert(node.id, RefCell::new(CHNode {
                neighbours: HashMap::new(),
                id: node.id,
                // l: 0,
                // houcount_sum: graph.get_outgoing_edges(node.id).len(),
                insertions: vec![],
                coordinades: Coordinates(node.lon, node.lat)
            }));
        }
        for edge in edges {
            let src_node = new_graph.nodes.get(&edge.src).unwrap();
            src_node.borrow_mut().neighbours.insert(edge.tgt, CHEdge { hopcount: 1, dist: edge.dist });
        }
        return new_graph;
    }

    pub fn get_print_nodes_and_edges(&self) -> (Vec<Node>, Vec<Edge>) {
        let mut nodes: Vec<Node> = self.nodes.iter().map(|e|  {
            let node = e.1.borrow();
            Node { id: node.id, lon: node.coordinades.0, lat: node.coordinades.1 }
        }).collect();
        nodes.sort_by(|e,f| e.id.cmp(&f.id));

        let mut edges = vec![];
        for node in &nodes {
            self.borrow_node(node.id).neighbours.iter().for_each(|(tgt, edge)| edges.push(Edge {src: node.id, tgt: *tgt, dist: edge.dist}));
        }
        return (nodes, edges);
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

    pub fn update_importance_of(
        &self, importance: &mut Vec<f32>, update_nodes: &HashSet<usize>, priority_queue: &mut BinaryMinHeap,
        l_counter: &Vec<usize>, ws_object: &mut Vec<HashMap<usize, u32>>, a_star_object: &AStartObject
    ) {
        let now = SystemTime::now();
        let mut witness_time = 0.0;
        let mut counter = 0;
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
                        let n1 = neighbour_ids[i];
                        let n2 = neighbour_ids[j];
                        let ws_now = SystemTime::now();
                        let is_shortcut_needed = self.is_shortcut_needed(*n1, *n2, edge_sum, *node_id, a_star_object);
                        witness_time += ws_now.elapsed().unwrap().as_secs_f32();
                        if is_shortcut_needed {
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
            counter += 1;
            if counter % 1000 == 0 {
                print!("\rUpdated importance of {} out of {} nodes", counter, update_nodes.len());
                let _ = io::stdout().flush();
            }
        }
        print!("\rUpdated importance of {} nodes in {} ms, ws: {} ms; ",
            update_nodes.len(), now.elapsed().unwrap().as_millis(), (witness_time * 1000.0) as u64
        );
    }

    #[inline]
    fn is_shortcut_needed(&self, n1: usize, n2: usize, edge_sum: u32, btw_node: usize, a_star_object: &AStartObject) -> bool {
        if self.borrow_node(n1).neighbours.contains_key(&n2) {
            return false;
        }
        let witness_search = ws_a_star(
            n1, n2, self, edge_sum, btw_node, a_star_object
        );
        return witness_search;
    }

    pub fn find_best_independent_set(&self, priority_queue: &mut BinaryMinHeap, importance: &Vec<f32>) -> (Vec<usize>, HashSet<usize>) {
        let mut independent_set: Vec<usize> = vec![];
        let mut neighbour_nodes: HashSet<usize> = HashSet::new();
        let mut importance_limit = f32::MAX;
        while !priority_queue.is_empty() {
            let next_node = priority_queue.pop(&importance);
            if importance[next_node] > importance_limit {
                break;
            }
            if !neighbour_nodes.contains(&next_node) {
                independent_set.push(next_node);
                neighbour_nodes.extend(self.borrow_node(next_node).neighbours.iter().map(|(tgt, _)| tgt));
            } else if importance_limit == f32::MAX {
                importance_limit = importance[next_node] + 1.0;
                // println!("Importance limit: {}", importance_limit);
            }
        }
        return (independent_set, neighbour_nodes);
    }

    pub fn contract_nodes(&mut self, nodes: &Vec<usize>, l_counter: &mut Vec<usize>, expansion_graph: &mut CHGraph) {
        nodes.iter().for_each(|node_id| {
            // println!("remove node {}", node_id);
            let removed_node = self.nodes.remove(node_id).expect(&format!("contraction node with id {} doesn't exist", node_id));

            // remove from neighbours and increment their l counter
            for (tgt, _) in &removed_node.borrow().neighbours {
                // println!("remove {} from {}", node_id, tgt);
                self.borrow_mut_node(*tgt).neighbours.remove(&node_id);
                l_counter[*tgt] += 1;
                expansion_graph.borrow_mut_node(*tgt).neighbours.remove(&node_id);      // because all neigbours will have higher level
            }

            // add insertions
            for (src, tgt, new_edge) in &removed_node.borrow().insertions {
                // println!("insert edge from {} to {}", src, tgt);
                self.borrow_mut_node(*tgt).neighbours.insert(*src, new_edge.clone());
                self.borrow_mut_node(*src).neighbours.insert(*tgt, new_edge.to_owned());
                expansion_graph.borrow_mut_node(*tgt).neighbours.insert(*src, new_edge.clone());
                expansion_graph.borrow_mut_node(*src).neighbours.insert(*tgt, new_edge.to_owned());
            }
        });
    }
    
    pub fn n_edges(&self) -> usize {
        self.nodes.iter().fold(0usize, |base,element| base + element.1.borrow().neighbours.len())
    }

    #[allow(dead_code)]
    pub fn nodes_and_edges_to_clipboard(&self, graph: &Graph, nodes: &Vec<usize>) {
        let mut the_string = String::new();
        the_string += "{\"type\": \"Feature\", \"properties\": {}, \"geometry\": {\"type\": \"MultiLineString\",\"coordinates\": [";
        the_string += &self.nodes.iter().map(|(src, node)| {
            node.borrow().neighbours.iter().map(|(tgt, _)| format!("[[{},{}],[{},{}]]", graph.nodes[*src].lon, graph.nodes[*src].lat, graph.nodes[*tgt].lon, graph.nodes[*tgt].lat)).reduce(|e,f| e + "," + &f).unwrap()
        }).reduce(|e,f| e + "," + &f).unwrap();
        the_string += "]}},";
        the_string += "{\"type\": \"Feature\", \"properties\": {}, \"geometry\": {\"type\": \"MultiPoint\",\"coordinates\": [";
        the_string += &nodes.iter().map(
            |node_id| format!("[{},{}]", graph.get_node(*node_id).lon, graph.get_node(*node_id).lat)
        ).reduce(|e,f| e+","+&f).unwrap();
        the_string += "]}}";
        cli_clipboard::set_contents(the_string.to_owned()).unwrap();
    }
}

pub struct CHNode {
    id: usize,
    pub neighbours: HashMap<usize, CHEdge>,
    // l: usize,
    // houcount_sum: usize,
    insertions: Insertions,
    pub coordinades: Coordinates
}

type Insertions = Vec<(usize, usize, CHEdge)>;

#[derive(Clone, Debug)]
pub struct CHEdge {pub hopcount: usize, pub dist: u32}