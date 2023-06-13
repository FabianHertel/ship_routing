use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::coordinates::distance_between;
use crate::coordinates::Coordinates;

use serde::{Serialize, Deserialize};

pub fn import_graph_from_file(path :&str) -> Result<Graph, std::io::Error>{

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();
    
    let mut num_nodes = 0;
    let mut num_edges = 0;

    let graph_file = File::open(path)?;
    let reader = BufReader::new(graph_file);

    let mut num_line = 0;

    for line in reader.lines() {
        let line = line?;
        line.split(" ");

        let numbers: Vec<f64> = line
        .split_whitespace()
        .map(|s| s.parse::<f64>())
        .collect::<Result<Vec<f64>, _>>()
        .unwrap_or_else(|_| {
             println!("Failed to parse numbers in line: {}", line); 
             Vec::new()
            });
        
        if(num_line == 0){
            num_nodes = numbers[0] as usize;
        }
        else if(num_line == 1){
            num_edges = numbers[0] as usize;
        } 
        else if (num_line > 1 && num_line < num_nodes + 2 ) {
            nodes.push(Node{
                id: numbers[0] as usize,
                lat: numbers[1],
                lon: numbers[2],
            })
        } 
        else if (num_line >= num_nodes + 2)  {
            edges.push(Edge { 
                src: numbers[0] as usize,
                tgt: numbers[1] as usize,
                dist: numbers[2]
             })
        }
        num_line += 1;
    }
    
    let mut next_src: usize = 0;
    let mut offset: usize = 0;
    let mut offsets = vec![0; num_nodes + 1];
    for edge in edges.iter() {
        if edge.src >= next_src {
            for j in next_src..=edge.src {
                offsets[j] = offset;
            }
            next_src = edge.src + 1;
        }
        offset += 1;
    }
    for i in next_src..= num_nodes {
        offsets[i] = num_edges;
    }
    
    println!("Finished importing");

    Ok(Graph {
        nodes : nodes,
        edges : edges,
        offsets: offsets,
    })
    
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