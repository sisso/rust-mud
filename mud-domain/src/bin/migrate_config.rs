use mud_domain::game::loader::{dto::ObjData, Loader};

use commons::tree::Tree;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::env;

use commons::{asciicolors, jsons::JsonValueExtra};
use std::path::Path;

fn usage() {
    println!();
    println!("Usage:");
    println!();
    println!("{} config-file", env::args().nth(0).unwrap());
    println!();
}

fn main() {
    if env::args().len() < 1 {
        usage();
        std::process::exit(1);
    }

    let path = env::args().nth(1).expect("config file is require");
    let path = Path::new(path.as_str());

    let mut data = if path.is_dir() {
        eprintln!("source config file can not be a directory");
        std::process::exit(2);
    } else if path.exists() {
        Loader::read_files(vec![path]).expect("fail to load file")
    } else {
        eprintln!("file or directory [{:?}] not found", path);
        std::process::exit(2);
    };

    // migrate it
    Loader::migrate(&mut data).expect("fail to migrate data");
    Loader::write_snapshot(path, &data).expect("fail to write config file");
    println!("done");
}
