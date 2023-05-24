use std::error::Error;
use std::time::{SystemTime};
use geojson::{Geometry, Value};
use std::fs;

mod read_pbf;

use crate::read_pbf::waypoints_coastline_parallel;

fn main() -> Result<(), Box<dyn Error>> {
    // Read command line argument and create IndexedReader
    let arg = std::env::args_os()
        .nth(1)
        .ok_or("need a *.osm.pbf file as argument")?;


    let now = SystemTime::now();
    println!("Importing pbf file...");
    let coastline = waypoints_coastline_parallel(&arg);
    println!("Ways with coastline tag:  {}", coastline.len());
    println!("Import and filter time: {}sek", now.elapsed()?.as_secs());

    let now = SystemTime::now();
    println!("Writing GeoJSON...");
    let geometry = Geometry::new(Value::MultiLineString(coastline));
    fs::write("./geojson.json", geometry.to_string()).expect("Unable to write file");
    println!("Time to write file: {}sek", now.elapsed()?.as_secs());

    Ok(())
}
