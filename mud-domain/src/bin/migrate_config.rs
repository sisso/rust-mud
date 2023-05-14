use clap;
use clap::ArgAction::SetTrue;
use mud_domain::game::loader::Loader;
use std::path::Path;

fn main() {
    // parse arguments
    let args = clap::Command::new("migrate config")
        .arg(
            clap::Arg::new("path")
                .help("path to the config file")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::new("dry-run")
                .help("do a dry run and output the result into the stdout")
                .short('d')
                .long("dry-run")
                .action(SetTrue),
        )
        .get_matches();

    let path = args.get_one::<String>("path").expect("path not provided");
    let dry = args.get_one::<bool>("dry-run").unwrap_or(&false);

    // read data
    let path = Path::new(path);
    if path.is_dir() {
        panic!("path can not be a dir")
    }
    if !path.exists() {
        panic!("path not found")
    }
    let mut data = Loader::read_json_from_file_raw(path).expect("fail to load file");

    // migrate it
    match Loader::migrate(&mut data) {
        Ok(_) => {
            if *dry {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).expect("fail to serialize")
                );
            } else {
                Loader::write_snapshot(path, &data).expect("fail to write config file");
            }
            println!("done");
        }
        Err(err) => {
            // let mut tmp_path = std::env::temp_dir();
            // tmp_path.push("rust_mud_migration_fail.json");

            eprintln!(
                "fail by {}, last data output",
                err,
                // tmp_path.display()
            );
            println!(
                "{}",
                serde_json::to_string_pretty(&data).expect("fail to serialize")
            );
            // Loader::write_snapshot(&tmp_path, &data).expect("fail to write config file");
        }
    }
}
