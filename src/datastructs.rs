
use std::f64::consts::PI;

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Coordinates(pub f64, pub f64);

impl Coordinates {
    pub fn from_vec(vector: &Vec<f64>) -> Coordinates {
        return Coordinates(vector[0], vector[1])
    }
    pub fn from_str(str: &str) -> Coordinates {
        let split:Vec<&str> = str.split(",").collect();
        return Coordinates(split[0].parse::<f64>().unwrap(), split[1].parse::<f64>().unwrap());
    }

    pub fn distance_to(&self, y: &Coordinates) -> f64 {
        return distance_between(self.0, self.1, y.0, y.1);
    }
}

pub fn distance_between(lon1:f64, lat1:f64, lon2:f64, lat2:f64) -> f64 {
    // from: http://www.movable-type.co.uk/scripts/latlong.html
    let φ1 = lat1 * PI/180.0; // φ, λ in radians
    let φ2 = lat2 * PI/180.0;
    let dφ = (lat2-lat1) * PI/180.0;
    let dλ = (lon2-lon1) * PI/180.0;
    const EARTH_RADIUS: f64 = 6371.0;

    let haversine = (dφ/2.0).sin().powi(2) + φ1.cos() * φ2.cos() * (dλ/2.0).sin().powi(2);
    let distance = EARTH_RADIUS * 2.0 * haversine.sqrt().atan2((1.0 - haversine).sqrt());
    return distance;
}

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.1)
    }
}

/// Result of a shortest path algorithm
pub struct ShortestPath(f64, Vec<Node>);

impl ShortestPath {
    /// Creates a new node result with distance `dist` and path `path` to the associated node
    pub fn new(dist: f64, path: Vec<Node>) -> Self {
        Self(dist, path)
    }

    /// Returns the distance to the associated node
    pub fn dist(&self) -> f64 {
        self.0
    }

    /// Returns the path from the source node to the associated node
    pub fn path(&self) -> &Vec<Node> {
        &self.1
    }
}

/// An undirected graph
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub offsets: Vec<usize>,
}

impl Graph {
    pub fn closest_node(&self, point: &Coordinates) -> &Node {
        let mut closest_node = &self.nodes[0];
        let mut closest_dist = f64::MAX;
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
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: usize,
    pub lon: f64,
    pub lat: f64
}

impl Node {
    pub fn distance_to(&self, y: &Coordinates) -> f64 {
        return distance_between(self.lon, self.lat, y.0, y.1);
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Edge {
    /// The id of the edge's source node
    pub src: usize,
    /// The id of the edge's target node
    pub tgt: usize,
    /// The edge's weight, i.e., the distance between its source and target
    pub dist: f64,
}
