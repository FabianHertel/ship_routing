use osmpbf::{Element, ElementReader};
use std::{ffi::OsString, collections::{HashSet, HashMap, LinkedList}, time::SystemTime, error::Error};
use geojson::{Geometry, Value};
use std::fs;
use rayon::prelude::*;


/* filters ways for tag coastline and then searches for coordinates of referenced nodes
 */
pub fn import_pbf(path: &OsString) -> Result<(), Box<dyn Error>> {
    println!("1/4: Read and filter OSM ways with tag 'coastline'...");
    let now = SystemTime::now();
    let waypoint_refs: Vec<Vec<i64>> = read_coastline(path);     //filter ways
    println!("1/4: Ways with coastline tag:  {}", waypoint_refs.len());
    println!("1/4: Finished in {} sek", now.elapsed()?.as_secs());

    println!("2/4: Connecting coastlines...");
    let now = SystemTime::now();
    let coastline = connect_coastlines(waypoint_refs);
    println!("2/4: Number of continents and islands:  {}", coastline.len());
    println!("2/4: Finished in {} sek", now.elapsed()?.as_secs());

    println!("3/4: Read coordinates of points and merge data...");
    let now = SystemTime::now();
    let coordinates = read_coordinates(path, &coastline);

    let coastline_coordinates: Vec<Vec<Vec<f64>>> = coastline.into_iter().map(|way| way.into_iter().map(|point_ref| {
        let coordinates = coordinates.get(&point_ref).unwrap();
        vec![coordinates.0, coordinates.1]
    }).collect()).collect();     // merge ways with coordinates and convert coordinates to vector
    println!("3/4: Finished in {} sek", now.elapsed()?.as_secs());
        
    println!("4/4: Write GeoJSON ...");
    let now = SystemTime::now();
    let geometry = Geometry::new(Value::MultiLineString(coastline_coordinates));
    fs::write("./geojson/import.json", geometry.to_string()).expect("Unable to write file");
    println!("4/4: Finished in {} sek", now.elapsed()?.as_secs());

    Ok(())
}

pub fn print_geojson(mut coastlines: Vec<Vec<Vec<f64>>>, prefix: &str) {
    coastlines.sort_by(|a,b| b.len().cmp(&a.len()));

    let continents = coastlines[..10].to_vec();
    let big_islands = coastlines[10..100].to_vec();
    let islands = coastlines[100..1000].to_vec();
    let small_islands = coastlines[1000..].to_vec();

    [("continents", continents), ("big_islands", big_islands), ("islands", islands), ("small_islands", small_islands)].par_iter().for_each(|file| {
        let filename = prefix.to_owned() + "_" + file.0;
        let now = SystemTime::now();
        let geometry_continents = Geometry::new(Value::MultiLineString(file.1.to_owned()));
        fs::write(format!("./geojson/{}.json", filename), geometry_continents.to_string()).expect("Unable to write file");
        println!("Finished {} in {} sek", filename, now.elapsed().unwrap().as_secs());
    });
}

/* reads and filters the ways with tag coastline
returns: list of references to nodes
 */
fn read_coastline(path: &OsString) -> Vec<Vec<i64>> {
    let reader = ElementReader::from_path(path);

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::Way(way) => {
                    if way.tags().any(|key_value| key_value == ("natural", "coastline")) {
                        LinkedList::from([way.refs().collect()])
                    } else {
                        LinkedList::new()
                    }
                },
                _ => LinkedList::new(),
            }
        },
        || LinkedList::new(),
        |mut a, mut b| {
            a.append(&mut b);
            a
        }
    ) {
        Ok(ways) => return ways.into_iter().collect(),
        Err(e) => {
            println!("{}", e.to_string());
            return vec![]
        }
    };
}


/* read coordinates of point ids from ways as vectors
 */
fn read_coordinates(path: &OsString, ways: &Vec<Vec<i64>>) -> HashMap<i64, (f64, f64)> {
    let reader = ElementReader::from_path(path);
    let node_set: HashSet<i64> = ways.to_owned().into_iter().reduce(|mut way_a, mut way_b| {
        way_a.append(&mut way_b);
        return way_a
    }).unwrap().into_iter().collect();   //refs of nodes into HashSet

    match reader.unwrap().par_map_reduce(   // TODO: try with vector instead HashMap and construct HashMap afterwards
        |element| {
            match element {
                Element::DenseNode(node) => {
                    if node_set.contains(&node.id) {    // add coordinates to loop vector which will be returned
                        LinkedList::from([(node.id, (node.lon(), node.lat()))])
                    } else {
                        LinkedList::new()
                    }
                },
                _ => LinkedList::new(),
            }
        },
        || LinkedList::new(),      // initial empty vector
        |mut a, mut b| {
            a.append(&mut b);         // merge vectors of parallel operations
            return a;
        }
    ) {
        Ok(ways) => return ways.into_iter().collect(),
        Err(e) => {
            println!("{}", e.to_string());
            return HashMap::new()
        }
    };
}


pub fn connect_coastlines(ways: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    let mut start_nodes: HashMap<i64, Vec<i64>> = HashMap::new();  // refers to incomplete_coastlines
    let mut end_nodes: HashMap<i64, Vec<i64>> = HashMap::new();    // refers to incomplete_coastlines
    let mut complete_coastlines: Vec<Vec<i64>> = vec![];    // contains only full coastlines, where the last point has the same id as the first

    for i in 0..ways.len() {
        if ways[i][0].to_owned() == ways[i][ways[i].len().to_owned()-1].to_owned() {
            complete_coastlines.push(ways[i].to_owned());
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
            }
            (None, Some(leading_coastline), None, None) => {
                new_coastline = leading_coastline;
                new_coastline.append(&mut ways[i][1..].to_vec());
            }
            (Some(following_coastline), Some(leading_coastline), None, None) => {
                if following_coastline[0] == leading_coastline[0] {
                    new_coastline = leading_coastline;
                    new_coastline.append(&mut ways[i][1..].to_vec());
                    complete_coastlines.push(new_coastline);
                    continue;
                }
                new_coastline = leading_coastline;
                new_coastline.append(&mut ways[i][1..].to_vec());
                new_coastline.append(&mut following_coastline[1..].to_vec());
            }
            (None, None, None, None) => {
                new_coastline = ways[i].to_owned();
            }
            (_, _, _, _) => {
                println!("Start-start or an end-end connection was encountered. Should not occur!")
            }
        }
        if new_coastline.len() > 1 {
            end_nodes.insert(*new_coastline.last().unwrap(), new_coastline.to_owned());
            start_nodes.insert(*new_coastline.first().unwrap(),new_coastline);
        }
    }
    assert_eq!(end_nodes.len(), 0);     // if not zero, there is a coastline where beginning and ending are not connected
    assert_eq!(start_nodes.len(), 0);   // if not zero, there is a coastline where beginning and ending are not connected
    return complete_coastlines
}