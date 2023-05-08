use osmpbf::{Element, ElementReader, IndexedReader};
use std::{ffi::OsString, collections::{HashSet, HashMap}};

/* filters ways for tag coastline and then searches for coordinates of referenced nodes
 */
pub fn waypoints_coastline_parallel(path: &OsString) -> Vec<Vec<Vec<f64>>> {
    let waypoint_refs: HashSet<Vec<i64>> = coastline_as_refs_parallel(path);     //filter ways
    let coordinates = coordinates_of_points(path, &waypoint_refs);

    waypoint_refs.into_iter().map(|way| way.into_iter().map(|point_ref| {
        let coordinates = coordinates.get(&point_ref).unwrap();
        vec![coordinates.0, coordinates.1]
    }).collect()).collect()     // merge ways with coordinates and convert coordinates to vector
}

/* read coordinates of point ids from ways as vectors
 */
fn coordinates_of_points(path: &OsString, ways: &HashSet<Vec<i64>>) -> HashMap<i64, (f64, f64)> {
    let reader = ElementReader::from_path(path);
    let node_set: HashSet<i64> = ways.to_owned().into_iter().reduce(|mut way_a, mut way_b| {
        way_a.append(&mut way_b);
        return way_a
    }).unwrap().into_iter().collect();   //refs of nodes into HashSet

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::DenseNode(node) => {
                    if node_set.contains(&node.id) {    // add coordinates to loop vector which will be returned
                        HashMap::from([(node.id, (node.lon(), node.lat()))])
                    } else {
                        HashMap::new()
                    }
                },
                _ => HashMap::new(),
            }
        },
        || HashMap::new(),      // initial empty vector
        |mut a, b| {
            a.extend(b);         // merge vectors of parallel operations
            return a;
        }
    ) {
        Ok(ways) => return ways.to_owned(),
        Err(e) => {
            println!("{}", e.to_string());
            return HashMap::new()
        }
    };
}

/* filters for the ways with tag coastline
returns: list of references to nodes
 */
fn coastline_as_refs_parallel(path: &OsString) -> HashSet<Vec<i64>> {
    let reader = ElementReader::from_path(path);

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::Way(way) => {
                    if way.tags().any(|key_value| key_value == ("natural", "coastline")) {
                        HashSet::from([way.refs().collect()])
                    } else {
                        HashSet::new()
                    }
                },
                _ => HashSet::new(),
            }
        },
        || HashSet::new(),      // Zero is the identity value for addition
        |mut a, b| {
            a.extend(b);
            return a
        }
    ) {
        Ok(ways) => return ways,
        Err(e) => {
            println!("{}", e.to_string());
            return HashSet::new()
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
                Element::Node(node) => nodes.push((node.lon(), node.lat())),
                Element::DenseNode(dense_node) => nodes.push((dense_node.lon(), dense_node.lat())),
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