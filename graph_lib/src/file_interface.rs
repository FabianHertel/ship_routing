use std::{io::{BufRead, BufReader, Write}, fs::File };

use crate::{Node, Edge, Graph};

pub fn import_graph_from_file(filename: &str) -> Result<Graph, std::io::Error> {
    let path = "data/graph/".to_owned() + filename;
    let bin_import = import_graph_from_bin_file(&(path.to_owned() + ".bin"));
    match bin_import {
        Ok(graph) => return Ok(graph),
        Err(_) => {
            println!("bin of graph not found, try fmi file");
            return import_graph_from_fmi_file(&(path.to_owned() + ".fmi"));
        },
    }
}

pub fn print_graph_to_file(nodes: &Vec<Node>, mut edges: &mut Vec<Edge>, filename: &str) {
    let path = "data/graph/".to_owned() + filename;

    print_bin_graph(nodes, edges, &(path.to_owned() + ".bin"));
    print_fmi_graph(nodes, &mut edges, &(path.to_owned() + ".fmi"));
}

/**
 * the filetype .bin will be added later
 */
fn print_bin_graph(points: &Vec<Node>, edges: &Vec<Edge>, filepath: &str) {
    let encoded = bincode::serialize(&(points, edges)).unwrap();
    
    let mut f = File::create(filepath).expect("Unable to create file");
    f.write_all(encoded.as_slice()).expect("unable to write file");
}

/**
 * the filetype .bin will be added later
 */
fn import_graph_from_bin_file(filepath: &str) -> Result<Graph, Box<bincode::ErrorKind>> {
    
    let graph_file = File::open(filepath)?;
    let reader = BufReader::new(graph_file);

    let decoded: Result<(Vec<Node>, Vec<Edge>), Box<bincode::ErrorKind>> = bincode::deserialize_from(reader);
    match decoded {
        Ok((nodes, edges)) => {
            let mut offsets = vec![0usize; nodes.len() + 1];
            let mut last_node = edges[0].src;
            for (i, edge) in edges.iter().enumerate() {
                if last_node != edge.src {
                    for node in (last_node+1)..(edge.src+1) {       // in case there is a node with 0 edges
                        offsets[node] = i;
                    }
                    last_node = edge.src;
                }
            }
            offsets[nodes.len()] = edges.len();
            return Ok(Graph { nodes, edges, offsets });
        },
        Err(error) => return Err(error),
    }
}

fn import_graph_from_fmi_file(filepath :&str) -> Result<Graph, std::io::Error> {

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    
    let mut num_nodes = 0;
    let mut num_edges = 0;

    let graph_file = File::open(filepath)?;
    let reader = BufReader::new(graph_file);

    let mut num_line = 0;

    for line in reader.lines() {
        let line = line?;
        line.split(" ");

        let numbers: Vec<f32> = line
        .split_whitespace()
        .map(|s| s.parse::<f32>())
        .collect::<Result<Vec<f32>, _>>()
        .unwrap_or_else(|_| {
             println!("Failed to parse numbers in line: {}", line); 
             Vec::new()
            });
        
        if num_line == 0 {
            num_nodes = numbers[0] as usize;
        }
        else if num_line == 1 {
            num_edges = numbers[0] as usize;
        } 
        else if num_line > 1 && num_line < num_nodes + 2  {
            nodes.push(Node{
                id: numbers[0] as usize,
                lat: numbers[1],
                lon: numbers[2],
            })
        } 
        else if num_line >= num_nodes + 2  {
            edges.push(Edge { 
                src: numbers[0] as usize,
                tgt: numbers[1] as usize,
                dist: numbers[2] as u32
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
    
    Ok(Graph {
        nodes : nodes,
        edges : edges,
        offsets: offsets,
    })
    
}


fn print_fmi_graph(points: &Vec<Node>, edges: &mut Vec<Edge>, filepath: &str) {
    let mut data_string = String::new();
    edges.sort_by(|a, b| a.src.cmp(&b.src));

    data_string = data_string + &points.len().to_string() + "\n";
    data_string = data_string + &edges.len().to_string() + "\n";

    for node in points {
        data_string = data_string
            + &node.id.to_string()
            + " "
            + &node.lat.to_string()
            + " "
            + &node.lon.to_string()
            + "\n";
    }
    for edge in edges {
        data_string = data_string
            + &edge.src.to_string()
            + " "
            + &edge.tgt.to_string()
            + " "
            + &edge.dist.to_string()
            + "\n";
    }

    let mut f = File::create(filepath).expect("Unable to create file");
    f.write_all(data_string.as_bytes()).expect("unable to write file");

}