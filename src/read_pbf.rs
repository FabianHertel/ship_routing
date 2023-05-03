use osmpbf::{Element, ElementReader, IndexedReader};
use std::{ffi::OsString, collections::{HashSet}};

pub fn waypoints_coastline_parallel(path: &OsString) -> Vec<(f64, f64)> {
    let reader = ElementReader::from_path(path);
    let waypoint_refs = par_coastline_as_refs(path);
    let node_set: HashSet<i64> = waypoint_refs.into_iter().collect();

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::DenseNode(node) => {
                    if node_set.contains(&node.id) {
                        vec![(node.lat(), node.lon())]
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
            a
        }   // Sum the partial results
    ) {
        Ok(ways) => return ways,
        Err(e) => {
            println!("{}", e.to_string());
            return vec![]
        }
    };
}

fn par_coastline_as_refs(path: &OsString) -> Vec<i64> {
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
            a
        }   // Sum the partial results
    ) {
        Ok(ways) => return ways.to_vec(),
        Err(e) => {
            println!("{}", e.to_string());
            return vec![]
        }
    };
}

#[allow(dead_code)]
pub fn waypoints_coastline_lib(path: &OsString) -> (i32, i32) {
    let mut ways = 0;
    let mut nodes = 0;

    let mut reader = IndexedReader::from_path(path).unwrap();

    match reader.read_ways_and_deps(
        |way| {
            // Filter ways. Return true if tags contain "building": "yes".
            way.tags().any(|key_value| key_value == ("natural", "coastline"));
            true
        },
        |element| {
            // Increment counter for ways and nodes
            match element {
                Element::Way(_way) => {
                    ways += 1
                },
                Element::Node(_node) => nodes += 1,
                Element::DenseNode(_dense_node) => nodes += 1,
                Element::Relation(_) => {} // should not occur
            }
        },
    ) {
        Ok(()) => return (nodes, ways),
        Err(e) => {
            println!("{}", e.to_string());
            return (0,0)
        }
    };
}