use std::error::Error;
use std::time::SystemTime;
use regex::Regex;

mod import_pbf;
mod generate_map;
mod island;
mod test_polygon_test;

use crate::import_pbf::{import_pbf, print_geojson};
use crate::generate_map::{generate_map, read_geojsons};
use crate::test_polygon_test::static_polygon_tests;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const COMMANDS: &str = "import/transform/generate/test";
    let command = std::env::args_os().nth(1)
        .ok_or(format!("need to specify the command, {}", COMMANDS))?;

    match command.to_str() {
        Some("import") => {
            let pbf_file = param_to_string(2, "planet.osm.pbf", Some(Regex::new(r"osm.pbf$")))?;
            let export_prefix = param_to_string(3, "complete", None)?;
    
            let now = SystemTime::now();
            println!("Importing pbf file...");
            import_pbf(&pbf_file, &export_prefix)?;
            println!("Import completed, overall time: {} sek", now.elapsed()?.as_secs());
        }
        Some("generate") => {
            let filename_out = param_to_string(2, "graph.fmi", Some(Regex::new(r".fmi$")))?;
            let import_prefix = param_to_string(3, "complete", None)?;
            
            let now = SystemTime::now();
            println!("Graph generation ...");
            generate_map(&filename_out, &import_prefix)?;
            println!("Graph generation completed, overall time: {} sek", now.elapsed()?.as_secs());
        }
        Some("transform") => {      // for developement
            let import_prefix = std::env::args_os().nth(2).ok_or("specify an import prefix")?;
            let export_prefix = std::env::args_os().nth(3).ok_or("specify an export prefix")?;
            let reduce = param_to_string(4, "complete", Some(Regex::new(r"true|false|f$")))?.trim() == "true";

            let now = SystemTime::now();
            println!("Transformation ...");
            transform(import_prefix.to_str().unwrap(), export_prefix.to_str().unwrap(), reduce)?;
            println!("Transformation completed, overall time: {} sek", now.elapsed()?.as_secs());
        }
        Some("test") => {       // for developement
            let import_prefix = param_to_string(2, "reduced", None)?;
            static_polygon_tests(&import_prefix);
        }
        Some(command) => println!("Command {} not known. Please specify one of {}", command, COMMANDS),
        None => panic!("Command is missing, but should not occur here!"),
    }

    Ok(())
}

fn param_to_string(nth: usize, alt_str: &str, regex: Option<Result<regex::Regex, regex::Error>>) -> Result<String, String> {
    match std::env::args_os().nth(nth) {
        Some(osstring) => {
            let param = osstring.into_string().unwrap();
            match regex {
                Some(regex) => {
                    let regex = regex.unwrap();
                    if !regex.is_match(&param) {return Err(format!("Parameter {} doesn't match format {}", param, regex))}
                }
                None => ()
            }
            return Ok(param)
        },
        None => return Ok(String::from(alt_str))
    };
}

fn transform(import_prefix: &str, export_prefix: &str, reduce: bool) -> Result<(), Box<dyn Error>> {
    println!("1/2: Read GeoJSONs parallel ...");
    let now = SystemTime::now();
    let coastlines: Vec<Vec<Vec<f32>>> = read_geojsons(import_prefix);
    println!("1/2: Finished in {} sek", now.elapsed()?.as_secs());
    
    println!("2/2: Write GeoJSON ...");
    let now = SystemTime::now();
    print_geojson(coastlines, export_prefix, reduce);
    println!("2/2: Finished in {} sek", now.elapsed()?.as_secs());

    Ok(())
}