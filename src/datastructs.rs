
pub struct Coordinates(pub f64, pub f64);

impl Coordinates {
    pub fn from_vec(vector: &Vec<f64>) -> Coordinates {
        return Coordinates(vector[0], vector[1])
    }
    pub fn from_str(str: &str) -> Coordinates {
        let split:Vec<&str> = str.split(",").collect();
        return Coordinates(split[0].parse::<f64>().unwrap(), split[1].parse::<f64>().unwrap());
    }
}

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.1)
    }
}

impl Clone for Coordinates {
    fn clone(&self) -> Self {
        Coordinates(self.0, self.1)
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

    /// Consumes the path from the source node to the associated node
    pub fn consume_path(self) -> Vec<Node> {
        self.1
    }
}

/// An undirected graph
pub struct Graph {
    nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub offsets: Vec<usize>,
}

impl Graph {
    pub fn closest_node(&self, coordinates: Coordinates) -> &Node {
        todo!()
    }
    
    /// Get the node with id `node_id`
    pub fn get_node(&self, node_id: usize) -> &Node {
        &self.nodes[node_id]
    }

    pub fn n_nodes(&self) -> usize {
        return self.nodes.len();
    }

    pub fn n_edges(&self) -> usize {
        return self.edges.len();
    }

    /// Get all outgoing edges of a particular node
    pub fn get_outgoing_edges(&self, node_id: usize) -> &[Edge] {
        &self.edges[self.offsets[node_id]..self.offsets[node_id + 1]]
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub lat: f64,
    pub lon: f64
}

#[derive(Clone, Copy)]
pub struct Edge {
    /// The id of the edge's source node
    pub src: usize,
    /// The id of the edge's target node
    pub tgt: usize,
    /// The edge's weight, i.e., the distance between its source and target
    pub dist: f64,
}