use std::error::Error;
use std::time::{SystemTime};

mod import_pbf;
mod generate_map;

use crate::import_pbf::import_pbf;
use crate::generate_map::generate_map;

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
