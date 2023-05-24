use osmpbf::{Element, ElementReader, IndexedReader};
use std::{ffi::OsString, collections::{HashSet, HashMap}, time::SystemTime};


/* filters ways for tag coastline and then searches for coordinates of referenced nodes
 */
pub fn waypoints_coastline_parallel(path: &OsString) -> Vec<Vec<Vec<f64>>> {
    let waypoint_refs: Vec<Vec<i64>> = coastline_as_refs_parallel(path);     //filter ways
    println!("Finished first read: ways with references in HashSet");

    let now = SystemTime::now();
    println!("Connecting coastlines...");
    let coastline = connect_coastlines(waypoint_refs);
    println!("Time to connect coastlines: {}sek", now.elapsed().unwrap().as_secs());

    let coordinates = coordinates_of_points(path, &coastline);
    println!("Finished second read: coordinates of all referenced points in HashMap");

    coastline.into_iter().map(|way| way.into_iter().map(|point_ref| {
        let coordinates = coordinates.get(&point_ref).unwrap();
        vec![coordinates.0, coordinates.1]
    }).collect()).collect()     // merge ways with coordinates and convert coordinates to vector
}

/* read coordinates of point ids from ways as vectors
 */
fn coordinates_of_points(path: &OsString, ways: &Vec<Vec<i64>>) -> HashMap<i64, (f64, f64)> {
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
fn coastline_as_refs_parallel(path: &OsString) -> Vec<Vec<i64>> {
    let reader = ElementReader::from_path(path);

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::Way(way) => {
                    if way.tags().any(|key_value| key_value == ("natural", "coastline")) {
                        vec![way.refs().collect()]
                    } else {
                        vec![]
                    }
                },
                _ => vec![],
            }
        },
        || vec![],      // Zero is the identity value for addition
        |mut a, mut b| {
            if a.len() == 0 {
                b
            } else if b.len() == 0 {
                a
            } else {
                b.append(&mut a);
                b
            }
        }
    ) {
        Ok(ways) => return ways,
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

pub fn connect_coastlines(mut ways: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    let mut start_nodes: HashMap<i64, &Vec<i64>> = HashMap::new();
    let mut end_nodes: HashMap<i64, &Vec<i64>> = HashMap::new();

    let mut m_count = 0;
    let mut n_count = 0;
    let mut se_count = 0;
    let mut es_count = 0;
    let mut other_count = 0;

    for i in 0..ways.len() {
        let end_connection = start_nodes.remove(&ways[i][ways[i].len()-1]);
        let start_connection = end_nodes.remove(&ways[i][0]);

        // should not occur
        let start_start_connection = start_nodes.remove(&ways[i][0]);
        let end_end_connection = end_nodes.remove(&ways[i][ways[i].len()-1]);
        
        let mut new_coastline: &Vec<i64> = &ways[i];

        match (end_connection, start_connection, end_end_connection, start_start_connection) {
            (Some(following_coastline), None, None, None) => {
                se_count +=1;
            }
            (None, Some(leading_coastline), None, None) => {
                es_count +=1;
            }
            (Some(following_coastline), Some(leading_coastline), None, None) => {
                m_count +=1;
            }
            (None, None, None, None) => {
                n_count +=1;
            }
            (_, _, _, _) => {
                other_count +=1;
                println!("Should not occur")
            }
        }
        end_nodes.insert(*new_coastline.last().unwrap(), &new_coastline);
        start_nodes.insert(*new_coastline.first().unwrap(), &new_coastline);
    }
    println!("Counts: {}, {}, {}, {}, {}", se_count, es_count, m_count, n_count, other_count);
    return ways
}