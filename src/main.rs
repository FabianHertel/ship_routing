use std::error::Error;
use std::time::{SystemTime};
use geojson::{GeoJson, Geometry, Value, Feature};
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
    println!("coastline:  {}", coastline.len());

    let geometry = Geometry::new(Value::MultiLineString(coastline));

    fs::write("./geojson.json", geometry.to_string()).expect("Unable to write file");

    match now.elapsed() {
        Ok(elapsed) => {println!("Import and filter time: {}sek", elapsed.as_secs());}
        Err(e) => {println!("Error: {e:?}");}
    }

    Ok(())
}
