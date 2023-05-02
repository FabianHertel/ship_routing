use std::error::Error;
use std::time::{SystemTime};

mod read_pbf;
use read_pbf::read_pbf_element_reader;
use read_pbf::read_pbf_index_reader;

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
