use clap;
use clap::ArgAction::SetTrue;
use mud_domain::game::loader::Loader;
use std::path::Path;

fn main() {
    // parse arguments
    let args = clap::Command::new("update config")
        .arg(
            clap::Arg::new("path")
                .help("path to the config file")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::new("dry-run")
                .help("dry run")
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
}
