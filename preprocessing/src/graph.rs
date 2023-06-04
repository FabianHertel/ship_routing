use std::fs::File;
use std::error::Error;

use std::io::{BufRead, BufReader};
use crate::datastructs::{Graph, Node, Edge};

pub fn import_graph_from_file(path :&str) -> Result<Graph, std::io::Error>{

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();
    
    let mut num_nodes = 0;
    let mut num_edges = 0;

    let graph_file = File::open(path)?;
    let mut reader = BufReader::new(graph_file);

    let mut num_line = 0;

    for line in reader.lines() {
        let line = line?;
        line.split(" ");

        let mut numbers: Vec<f64> = line
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
                lon: numbers[1],
                lat: numbers[2],
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
    
    let mut offset: usize = 0;
    for i in 0..nodes.len(){
        offsets.push(offset);
        offset += edges.iter().filter(|e| e.src == i).count();       
    }
    
    // println!("{}", num_nodes);
    // println!("{}", num_edges);
    // println!("{:?}", nodes);
    // println!("{:?}", edges);
    // print!("{:?}", offsets);
    
    Ok(Graph {
        nodes : nodes,
        edges : edges,
        offsets: offsets,
    })
    
}