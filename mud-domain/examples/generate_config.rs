use mud_domain::game::loader::{Loader, ObjData};

use commons::tree::Tree;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn usage() {
    println!();
    println!("Usage:");
    println!();
    println!(
        "{} config-folder-raw config-folder",
        env::args().nth(0).unwrap()
    );
    println!();
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let (from_folder, to_folder) = match args.len() {
        3 => (&args[1], &args[2]),

        _ => {
            eprintln!("invalid arguments {:?}", args);

            usage();
            std::process::exit(1);
        }
    };

    clean_up(to_folder)?;
    generate(from_folder, to_folder);
    Ok(())
}

fn clean_up(folder: &str) -> Result<(), Box<dyn Error>> {
    let list: std::fs::ReadDir = std::fs::read_dir(folder)?;

    for entry in list {
        let path = entry?.path();
        if path.ends_with(".json") {
            print!("removing {:?}", path);
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn generate(path: &str, to_folder: &str) -> Result<(), Box<dyn Error>> {
    let data = Loader::read_folders(Path::new(path))?;
    let mut file = std::fs::File::create(Path::new(to_folder).join("default.json"))?;
    let json_str = serde_json::to_string_pretty(&data)?;
    file.write_all(json_str.as_bytes())?;
    Ok(())
}
