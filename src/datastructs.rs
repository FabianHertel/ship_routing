
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
pub struct ShortestPath<'a>(usize, Vec<&'a Node>);

impl<'a> ShortestPath<'a> {
    /// Creates a new node result with distance `dist` and path `path` to the associated node
    pub fn new(dist: usize, path: Vec<&'a Node>) -> Self {
        Self(dist, path)
    }

    /// Returns the distance to the associated node
    pub fn dist(&self) -> usize {
        self.0
    }

    /// Returns the path from the source node to the associated node
    pub fn path(&self) -> &Vec<&'a Node> {
        &self.1
    }

    /// Consumes the path from the source node to the associated node
    pub fn consume_path(self) -> Vec<&'a Node> {
        self.1
    }
}

/// An undirected graph
pub struct Graph {
    nodes: Vec<Node>,
    pub edges: Vec<Edge>
}
impl Graph {
    pub(crate) fn closest_node(&self, coordinates: Coordinates) -> &Node {
        todo!()
    }
}

#[derive(Debug)]
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