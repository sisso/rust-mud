use mud_domain::game::loader::{Loader};




use clap;

use std::path::Path;

fn main() {
    let matches = clap::App::new("update config")
        .arg(
            clap::Arg::new("path")
                .about("path to the config file")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::new("dry")
                .short('d')
                .long("dry-run")
                .about("dry run"),
        )
        .get_matches();

    let path = matches.value_of("path").unwrap();
    let dry = matches.is_present("dry");

    let path = Path::new(path);

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
    if !dry {
        Loader::write_snapshot(path, &data).expect("fail to write config file");
    }
    println!("done");
}
