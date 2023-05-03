use osmpbf::{Element, ElementReader, IndexedReader};
use std::{ffi::OsString, collections::HashSet};

/* filters ways for tag coastline and then searches for coordinates of referenced nodes
 */
pub fn waypoints_coastline_parallel(path: &OsString) -> Vec<(f64, f64)> {
    let reader = ElementReader::from_path(path);
    let waypoint_refs: Vec<i64> = coastline_as_refs_parallel(path);     //filter ways
    let node_set: HashSet<i64> = waypoint_refs.into_iter().collect();   //refs of nodes into HashSet

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::DenseNode(node) => {
                    if node_set.contains(&node.id) {    // add coordinates to loop vector which will be returned
                        vec![(node.lat(), node.lon())]
                    } else {
                        vec![]
                    }
                },
                _ => vec![],
            }
        },
        || [].to_vec(),      // initial empty vector
        |mut a, mut b| {
            a.append(&mut b);         // merge vectors of parallel operations
            return a
        }
    ) {
        Ok(ways) => return ways,
        Err(e) => {
            println!("{}", e.to_string());
            return vec![]
        }
    };
}

/* filters for the ways with tag coastline
returns: list of references to nodes
 */
fn coastline_as_refs_parallel(path: &OsString) -> Vec<i64> {
    let reader = ElementReader::from_path(path);

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::Way(way) => {
                    if way.tags().any(|key_value| key_value == ("natural", "coastline")) {
                        way.refs().collect()
                    } else {
                        vec![]
                    }
                },
                _ => vec![],
            }
        },
        || [].to_vec(),      // Zero is the identity value for addition
        |mut a, mut b| {
            a.append(&mut b);
            return a
        }
    ) {
        Ok(ways) => return ways.to_vec(),
        Err(e) => {
            println!("{}", e.to_string());
            return vec![]
        }
    };
}

#[allow(dead_code)]
pub fn waypoints_coastline_lib(path: &OsString) -> Vec<(f64, f64)> {
    let mut nodes: Vec<(f64, f64)> = vec![];
    let mut reader = IndexedReader::from_path(path).unwrap();

    match reader.read_ways_and_deps(
        |way| {
            // Filter ways for coastline
            way.tags().any(|key_value| key_value == ("natural", "coastline"))
        },
        |element| {
            // add nodes to list
            match element {
                Element::Node(node) => nodes.push((node.lat(), node.lon())),
                Element::DenseNode(dense_node) => nodes.push((dense_node.lat(), dense_node.lon())),
                _ => {}
            }
        },
    ) {
        Ok(()) => return nodes,
        Err(e) => {
            println!("{}", e.to_string());
            return vec![]
        }
    };
}