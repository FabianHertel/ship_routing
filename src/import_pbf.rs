use osmpbf::{Element, ElementReader, IndexedReader};
use std::{ffi::OsString, collections::{HashSet, HashMap}, time::SystemTime, error::Error};
use geojson::{Geometry, Value};
use std::fs;


/* filters ways for tag coastline and then searches for coordinates of referenced nodes
 */
pub fn import_pbf(path: &OsString) -> Result<(), Box<dyn Error>> {
    println!("1/4: Read and filter OSM ways with tag 'coastline'...");
    let now = SystemTime::now();
    let waypoint_refs: Vec<Vec<i64>> = coastline_as_refs_parallel(path);     //filter ways
    println!("1/4: Ways with coastline tag:  {}", waypoint_refs.len());
    println!("1/4: Finished in {}sek", now.elapsed()?.as_secs());

    println!("2/4: Connecting coastlines...");
    let now = SystemTime::now();
    let coastline = connect_coastlines(waypoint_refs);
    println!("2/4: N of continents and islands:  {}", coastline.len());
    println!("2/4: Finished in {}sek", now.elapsed()?.as_secs());

    println!("3/4: Read coordinates of points and merge data...");
    let now = SystemTime::now();
    let coordinates = coordinates_of_points(path, &coastline);

    let coastline_coordinates = coastline.into_iter().map(|way| way.into_iter().map(|point_ref| {
        let coordinates = coordinates.get(&point_ref).unwrap();
        vec![coordinates.0, coordinates.1]
    }).collect()).collect();     // merge ways with coordinates and convert coordinates to vector
    println!("3/4: Finished in {}sek", now.elapsed()?.as_secs());
    
        
    println!("4/4: Write GeoJSON ...");
    let now = SystemTime::now();
    let geometry = Geometry::new(Value::MultiLineString(coastline_coordinates));
    fs::write("./geojson.json", geometry.to_string()).expect("Unable to write file");
    println!("4/4: Finished in {}sek", now.elapsed()?.as_secs());

    Ok(())
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

pub fn connect_coastlines(ways: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    let mut start_nodes: HashMap<i64, Vec<i64>> = HashMap::new();  // refers to incomplete_coastlines
    let mut end_nodes: HashMap<i64, Vec<i64>> = HashMap::new();    // refers to incomplete_coastlines
    let mut complete_coastlines: Vec<Vec<i64>> = vec![];    // contains only full coastlines, where the last point has the same id as the first

    let mut m_count = 0;
    let mut n_count = 0;
    let mut c_count = 0;
    let mut se_count = 0;
    let mut es_count = 0;
    let mut f_count = 0;
    let mut other_count = 0;

    for i in 0..ways.len() {
        if ways[i][0].to_owned() == ways[i][ways[i].len().to_owned()-1].to_owned() {
            complete_coastlines.push(ways[i].to_owned());
            c_count += 1;
            continue;
        }
        let end_connection = start_nodes.remove(&ways[i][ways[i].len()-1]);
        let start_connection = end_nodes.remove(&ways[i][0]);

        // should not occur
        let start_start_connection = start_nodes.remove(&ways[i][0]);
        let end_end_connection = end_nodes.remove(&ways[i][ways[i].len()-1]);
        
        let mut new_coastline: Vec<i64> = vec![];

        match (end_connection, start_connection, end_end_connection, start_start_connection) {
            (Some(following_coastline), None, None, None) => {
                new_coastline = ways[i].to_owned();
                new_coastline.append(&mut following_coastline[1..].to_vec());
                se_count +=1;
            }
            (None, Some(leading_coastline), None, None) => {
                new_coastline = leading_coastline;
                new_coastline.append(&mut ways[i][1..].to_vec());
                es_count +=1;
            }
            (Some(following_coastline), Some(leading_coastline), None, None) => {
                if following_coastline[0] == leading_coastline[0] {
                    new_coastline = leading_coastline;
                    new_coastline.append(&mut ways[i][1..].to_vec());
                    complete_coastlines.push(new_coastline);
                    f_count += 1;
                    continue;
                }
                new_coastline = leading_coastline;
                new_coastline.append(&mut ways[i][1..].to_vec());
                new_coastline.append(&mut following_coastline[1..].to_vec());
                m_count +=1;
            }
            (None, None, None, None) => {
                new_coastline = ways[i].to_owned();
                n_count +=1;
            }
            (_, _, _, _) => {
                other_count +=1;
                println!("Should not occur")
            }
        }
        if new_coastline.len() > 1 {
            end_nodes.insert(*new_coastline.last().unwrap(), new_coastline.to_owned());
            start_nodes.insert(*new_coastline.first().unwrap(),new_coastline);
        }
    }
    println!("Counts: {}, {}, {}, {}, {}, {}, {}", se_count, es_count, m_count, n_count, c_count, f_count, other_count);
    return complete_coastlines
}