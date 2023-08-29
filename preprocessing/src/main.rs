use std::collections::HashSet;
use std::error::Error;
use std::time::SystemTime;
use graph_lib::{import_graph_from_file, Coordinates, Graph, Node};
use regex::Regex;

mod import_pbf;
mod generate_graph;
mod island;
mod test_polygon_test;

use crate::import_pbf::{import_pbf, print_geojson};
use crate::generate_graph::{generate_graph, read_geojsons, print_fmi_graph};
use crate::test_polygon_test::static_polygon_tests;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const COMMANDS: &str = "import/transform/generate/test";
    let command = std::env::args_os().nth(1)
        .ok_or(format!("need to specify the command, {}", COMMANDS))?;

    match command.to_str() {
        Some("import") => {
            let pbf_file = param_to_string(2, "planet.osm.pbf", Some(Regex::new(r"osm.pbf$")))?;
            let export_prefix = param_to_string(3, "complete", None)?;
    
            let now = SystemTime::now();
            println!("Importing pbf file...");
            import_pbf(&pbf_file, &export_prefix)?;
            println!("Import completed, overall time: {} sek", now.elapsed()?.as_secs());
        }
        Some("generate") => {
            let filename_out = param_to_string(2, "graph.fmi", Some(Regex::new(r".fmi$")))?;
            let import_prefix = param_to_string(3, "complete", None)?;
            
            let now = SystemTime::now();
            println!("Graph generation ...");
            generate_graph(&filename_out, &import_prefix)?;
            println!("Graph generation completed, overall time: {} sek", now.elapsed()?.as_secs());
        }
        Some("transform") => {      // for developement
            let import_prefix = std::env::args_os().nth(2).ok_or("specify an import prefix")?;
            let export_prefix = std::env::args_os().nth(3).ok_or("specify an export prefix")?;
            let reduce = param_to_string(4, "complete", Some(Regex::new(r"true|false|f$")))?.trim() == "true";

            let now = SystemTime::now();
            println!("Transformation ...");
            transform(import_prefix.to_str().unwrap(), export_prefix.to_str().unwrap(), reduce)?;
            println!("Transformation completed, overall time: {} sek", now.elapsed()?.as_secs());
        }
        Some("black_sea") => {
            println!("Importing fmi file...");
            let graph = import_graph_from_file("./data/graph-4mio-complete-u32.fmi").expect("Error importing Graph");
            let in_black_sea = Coordinates(31.80666484258255, 44.0467677172621);
            let node_in_black_sea = graph.closest_node(&in_black_sea);
            sub_graph_connected(&graph, node_in_black_sea);
        }
        Some("test") => {       // for developement
            let import_prefix = param_to_string(2, "reduced", None)?;
            static_polygon_tests(&import_prefix);
        }
        Some(command) => println!("Command {} not known. Please specify one of {}", command, COMMANDS),
        None => panic!("Command is missing, but should not occur here!"),
    }

    Ok(())
}

fn sub_graph_connected(graph: &Graph, node_in_black_sea: &Node) {
    let mut nodes_to_check: Vec<usize> = graph.get_outgoing_edges(node_in_black_sea.id).to_vec().iter().map(|e| e.tgt).collect();
    let mut found_nodes: HashSet<usize> = nodes_to_check.clone().into_iter().collect();

    while nodes_to_check.len() > 1 {
        let node = nodes_to_check.pop().unwrap();
        for edge in graph.get_outgoing_edges(node) {
            if !found_nodes.contains(&edge.tgt) {
                found_nodes.insert(edge.tgt);
                nodes_to_check.push(edge.tgt);
            }
        }
    }

    let mut black_sea = graph.subgraph(found_nodes.into_iter().collect());
    println!("nodes: {}, edges: {}", black_sea.n_nodes(), black_sea.edges.len());
    print_fmi_graph(&black_sea.nodes, &mut black_sea.edges, "black_sea.fmi");
}

fn param_to_string(nth: usize, alt_str: &str, regex: Option<Result<regex::Regex, regex::Error>>) -> Result<String, String> {
    match std::env::args_os().nth(nth) {
        Some(osstring) => {
            let param = osstring.into_string().unwrap();
            match regex {
                Some(regex) => {
                    let regex = regex.unwrap();
                    if !regex.is_match(&param) {return Err(format!("Parameter {} doesn't match format {}", param, regex))}
                }
                None => ()
            }
            return Ok(param)
        },
        None => return Ok(String::from(alt_str))
    };
}

fn transform(import_prefix: &str, export_prefix: &str, reduce: bool) -> Result<(), Box<dyn Error>> {
    println!("1/2: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f32>>> = read_geojsons(import_prefix);
    println!("1/2: Finished in {} sek", now.elapsed()?.as_secs());
    
    println!("2/2: Write GeoJSON ...");
    let now = SystemTime::now();
    print_geojson(coastlines, export_prefix, reduce);
    println!("2/2: Finished in {} sek", now.elapsed()?.as_secs());

    Ok(())
}