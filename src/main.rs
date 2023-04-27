use osmpbf::{Element, ElementReader, IndexedReader};
use std::error::Error;
use std::ffi::OsString;
use std::time::{SystemTime};

fn main() -> Result<(), Box<dyn Error>> {
    // Read command line argument and create IndexedReader
    let arg = std::env::args_os()
        .nth(1)
        .ok_or("need a *.osm.pbf file as argument")?;


    let now = SystemTime::now();
    println!("Counting...");

    let (nodes, ways) = read_pbf_index_reader(&arg);
    println!("ways:  {}\nnodes: {}", ways, nodes);

    let coastline = read_pbf_element_reader(&arg);
    println!("coastline:  {}", coastline.len());

    match now.elapsed() {
        Ok(elapsed) => {
            println!("{}", elapsed.as_secs());
        }
        Err(e) => {
            println!("Error: {e:?}");
        }
    }

    Ok(())
}

fn read_pbf_element_reader(path: &OsString) -> Vec<(f64, f64)> {
    let reader = ElementReader::from_path(path);

    match reader.unwrap().par_map_reduce(
        |element| {
            match element {
                Element::Node(node) => [(node.lat(), node.lon())].to_vec(),
                Element::DenseNode(node) => {
                    if node.tags().any(|key_value| key_value == ("natural", "coastline")) {
                        [(node.lat(), node.lon())].to_vec()
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

fn read_pbf_index_reader(path: &OsString) -> (i32, i32) {
    let mut ways = 0;
    let mut nodes = 0;

    let mut reader = IndexedReader::from_path(path).unwrap();

    match reader.read_ways_and_deps(
        |way| {
            // Filter ways. Return true if tags contain "building": "yes".
            way.tags().any(|key_value| key_value == ("natural", "coastline"))
        },
        |element| {
            // Increment counter for ways and nodes
            match element {
                Element::Way(_way) => ways += 1,
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