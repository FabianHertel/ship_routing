use std::error::Error;
use std::time::{SystemTime};

mod import_pbf;
mod generate_map;
mod dijkstra;
mod datastructs;
mod binary_minheap_alex;
mod graph;

use crate::datastructs::{Coordinates};
use crate::dijkstra::run_dijkstra;
use crate::graph::import_graph_from_file;
use crate::import_pbf::{import_pbf, print_geojson};
use crate::generate_map::{generate_map, read_geojsons};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const COMMANDS: &str = "import/generate/run";

    let command = std::env::args_os()
        .nth(1)
        .ok_or(format!("need to specify the command, {}", COMMANDS))?;

    match command.to_str() {
        Some("import") => {
            let arg = std::env::args_os()
                .nth(2)
                .ok_or("need a *.osm.pbf file as argument")?;
    
            let now = SystemTime::now();
            println!("Importing pbf file...");
            import_pbf(&arg)?;
            println!("Import completed, overall time: {}sek", now.elapsed()?.as_secs());
        }
        Some("transform") => {      // for developement; probably removed in production
            let import_prefix = std::env::args_os().nth(2)
                .ok_or("specify an import prefix")?;
            let export_prefix = std::env::args_os().nth(3)
                .ok_or("specify an export prefix")?;
            let reduce = std::env::args_os().nth(4)
                .ok_or("define, if you want to reduce the data")?.to_str().unwrap().trim() == "true";
            
            println!("1/2: Read GeoJSONs parallel ...");
            let now = SystemTime::now();
            let coastlines: Vec<Vec<Vec<f64>>> = read_geojsons(import_prefix.to_str().unwrap());
            println!("1/2: Finished in {} sek", now.elapsed()?.as_secs());
            
            println!("2/2: Write GeoJSON ...");
            let now = SystemTime::now();
            print_geojson(coastlines, export_prefix.to_str().unwrap(), reduce);
            println!("2/2: Finished in {} sek", now.elapsed()?.as_secs());
        }
        Some("generate") => {
            generate_map()?;
        }
        Some("dijkstra") => {
            let source_coordinates_str = std::env::args_os().nth(2)
                .ok_or("specify start coodinates lon,lat")?;
            let target_coordinates_str = std::env::args_os().nth(3)
                .ok_or("specify end coodinates lon,lat")?;
            let source_coordinats = Coordinates::from_str(source_coordinates_str.to_str().unwrap());
            let target_coordinats = Coordinates::from_str(target_coordinates_str.to_str().unwrap());
            match run_dijkstra(source_coordinats, target_coordinats)? {
                Some(path) => println!("Pfad: {:?}, Länge: {}", path.path(), path.dist()),
                None => println!("No path found")
            }
        }

        Some(command) => println!("Command {} not known. Please specify one of {}", command, COMMANDS),
        None => println!("need to specify the command, {}", COMMANDS),
    }

    Ok(())
}
