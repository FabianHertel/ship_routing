pub mod file_interface;
pub mod random_point;
pub mod island;
pub mod test_polygon_test;

use std::{f32::consts::PI, fmt::{Display, Formatter, Debug}, collections::HashMap};
use serde::{Serialize, Deserialize};
use cli_clipboard;

/**
 * (lon, lat)
 */
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Coordinates(pub f32, pub f32);

impl Coordinates {
    pub fn from_vec(vector: &Vec<f32>) -> Coordinates {
        return Coordinates(vector[0], vector[1])
    }
    pub fn from_str(str: &str) -> Coordinates {
        let split:Vec<&str> = str.split(",").collect();
        return Coordinates(split[0].parse::<f32>().unwrap(), split[1].parse::<f32>().unwrap());
    }

    #[inline]
    pub fn distance_to(&self, y: &Coordinates) -> f32 {
        return distance_between(self.0, self.1, y.0, y.1);
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.1)
    }
}

/**
 * direct distance in m as f32 on surface
 */
#[inline]
pub fn distance_between(lon1:f32, lat1:f32, lon2:f32, lat2:f32) -> f32 {
    // from: http://www.movable-type.co.uk/scripts/latlong.html
    let φ1 = lat1 * PI/180.0; // φ, λ in radians
    let φ2 = lat2 * PI/180.0;
    let dφ = (lat2-lat1) * PI/180.0;
    let dλ = (lon2-lon1) * PI/180.0;
    const EARTH_RADIUS: f32 = 6371.0;

    let haversine = (dφ/2.0).sin().powi(2) + φ1.cos() * φ2.cos() * (dλ/2.0).sin().powi(2);
    let distance = EARTH_RADIUS * 2000.0 * haversine.sqrt().atan2((1.0 - haversine).sqrt());
    return distance;
}

/**
 * undirected graph, saved in vectors of nodes, edges and offsets
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub offsets: Vec<usize>,
}

impl Graph {
    pub fn clone(&self) -> Graph {
        Graph { nodes: self.nodes.clone(), edges: self.edges.clone(), offsets: self.offsets.clone() }
    }
    
    /**
     * needed to find starting node
     */
    pub fn closest_node(&self, point: &Coordinates) -> &Node {
        let mut closest_node = &self.nodes[0];
        let mut closest_dist = f32::MAX;
        for node in &self.nodes {
            let distance = node.distance_to(point);
            if distance < closest_dist {
                closest_node = node;
                closest_dist = distance;
            }
        }
        return closest_node;
    }
    
    /// Get the node with id `node_id`
    pub fn get_node(&self, node_id: usize) -> &Node {
        &self.nodes[node_id]
    }

    pub fn n_nodes(&self) -> usize {
        return self.nodes.len();
    }

    #[allow(dead_code)]
    pub fn n_edges(&self) -> usize {
        return self.edges.len();
    }

    /// Get all outgoing edges of a particular node
    pub fn get_outgoing_edges(&self, node_id: usize) -> &[Edge] {
        &self.edges[self.offsets[node_id]..self.offsets[node_id + 1]]
    }

    /**
     * generates a subgraph, which contains only the nodes with given ids
     * needed for development and testing
     */
    pub fn subgraph(&self, node_list: Vec<usize>) -> Graph {
        let mut new_nodes: Vec<Node> = vec![];
        let mut new_edges: Vec<Edge> = vec![];
        let mut new_offsets: Vec<usize> = vec![];

        let mut map_old_id_to_new_id: HashMap<usize, usize> = HashMap::new();
        for (i, node_id) in node_list.iter().enumerate() {
            map_old_id_to_new_id.insert(*node_id, i);
        }

        let mut offset_counter = 0;
        for (new_id, old_id) in node_list.into_iter().enumerate() {
            let old_node = self.get_node(old_id);
            new_nodes.push(Node { id: new_id, lon: old_node.lon, lat: old_node.lat });
            let old_edges = self.get_outgoing_edges(old_id);
            new_edges.extend(old_edges.iter().map(|edge| Edge {src: new_id, tgt: map_old_id_to_new_id[&edge.tgt], dist: edge.dist}));
            new_offsets.push(offset_counter);
            offset_counter += old_edges.len();
        }

        return Graph { nodes: new_nodes, edges: new_edges, offsets: new_offsets }
    }

    /**
     * useful for displaying graph on geojson.io
     */
    pub fn nodes_to_clipboard(&self) {
        let mut the_string = "[".to_string();
        the_string += &self.nodes.iter().map(|node| format!("[{},{}]", node.lon, node.lat)).reduce(|e,f| e + "," + &f).unwrap();
        the_string += "]";
        cli_clipboard::set_contents(the_string.to_owned()).unwrap();
        assert_eq!(cli_clipboard::get_contents().unwrap(), the_string);
    }

    /**
     * useful for displaying graph on geojson.io
     */
    pub fn edges_to_clipboard(&self) {
        let mut the_string = "[".to_string();
        the_string += &self.edges.iter().map(|edge| format!("[[{},{}],[{},{}]]", self.nodes[edge.src].lon, self.nodes[edge.src].lat, self.nodes[edge.tgt].lon, self.nodes[edge.tgt].lat)).reduce(|e,f| e + "," + &f).unwrap();
        the_string += "]";
        cli_clipboard::set_contents(the_string.to_owned()).unwrap();
        assert_eq!(cli_clipboard::get_contents().unwrap(), the_string);
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: usize,
    pub lon: f32,
    pub lat: f32
}

impl Node {
    pub fn distance_to(&self, y: &Coordinates) -> f32 {
        return distance_between(self.lon, self.lat, y.0, y.1);
    }
    pub fn distance_to_node(&self, y: &Node) -> f32 {
        return distance_between(self.lon, self.lat, y.lon, y.lat);
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Edge {
    /// The id of the edge's source node
    pub src: usize,
    /// The id of the edge's target node
    pub tgt: usize,
    /// The edge's weight, i.e., the distance between its source and target
    pub dist: u32,
}


/// Result of a shortest path algorithm
pub struct ShortestPathResult {
    pub distance: u32,
    pub path: Option<Vec<Node>>,
    pub calculation_time: u128,
    pub visited_nodes: u32
}

impl Display for ShortestPathResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.path.as_ref() {
            Some(path) => 
            write!(f, "Result: {} km over {} nodes by checking {} nodes in {} millis",
                self.distance as f32 / 1000.0, path.len(),
                self.visited_nodes, self.calculation_time),
            None => write!(f, "NO RESULT by checking {} nodes in {} millis",
                    self.visited_nodes, self.calculation_time)
        }
    }
}

impl Debug for ShortestPathResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.path.as_ref() {
            Some(path) => 
                write!(f, "Result: {} km over {:?} by checking {} nodes in {} millis",
                    self.distance as f32 / 1000.0, path,
                    self.visited_nodes, self.calculation_time),
            None => write!(f, "NO RESULT by checking {} nodes in {} millis",
                    self.visited_nodes, self.calculation_time)
        }
    }
}