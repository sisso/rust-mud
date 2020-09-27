use mud_domain::game::loader::Loader;

use serde_json;

use std::env;
use std::error::Error;

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

/// Generate json config from HOCON
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
    generate(from_folder, to_folder)
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
    let path = Path::new(to_folder).join("default.json");
    Loader::write_snapshot(&path, &data)?;
    Ok(())
}
