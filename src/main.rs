use std::error::Error;
use std::time::{SystemTime};
use rayon::prelude::*;

mod import_pbf;
mod generate_map;

use crate::import_pbf::{import_pbf, print_geojson};
use crate::generate_map::{generate_map, read_geojsons};

fn main() -> Result<(), Box<dyn Error>> {
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
                .ok_or("define, if you want to reduce the data")?;
            
            println!("1/2: Read GeoJSONs parallel ...");
            let now = SystemTime::now();
            let mut coastlines: Vec<Vec<Vec<f64>>> = read_geojsons(import_prefix.to_str().unwrap());
            println!("1/2: Finished in {} sek", now.elapsed()?.as_secs());
            
            if reduce.to_str().unwrap().trim() == "true" {
                coastlines = reduces_coastlines(coastlines);
            }
        
            println!("2/2: Write GeoJSON ...");
            let now = SystemTime::now();
            print_geojson(coastlines, export_prefix.to_str().unwrap());
            println!("2/2: Finished in {} sek", now.elapsed()?.as_secs());
        }
        Some("generate") => {
            generate_map()?;
        }
        Some("run") => {
            todo!()
        }
        Some(command) => println!("Command {} not known. Please specify one of {}", command, COMMANDS),
        None => println!("need to specify the command, {}", COMMANDS),
    }

    Ok(())
}

fn reduces_coastlines(mut coastlines: Vec<Vec<Vec<f64>>>) -> Vec<Vec<Vec<f64>>> {
    return coastlines.par_iter_mut().filter(|a| a.len() > 400).map(|a| {
            let mut reduced_line: Vec<Vec<f64>> = a.iter().step_by(100).map(|a| a.to_owned()).collect();
            reduced_line.push(a.last().unwrap().to_owned());
            return reduced_line;
    }).collect()
}