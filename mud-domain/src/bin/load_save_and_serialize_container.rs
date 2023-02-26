use mud_domain::game::container::Container;
use mud_domain::game::loader;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let mut c = Container::new();
    let path = commons::io::search_file_backwards_deep(&env::current_dir().unwrap(), |str| {
        str.ends_with("data/space")
    })
    .unwrap();

    loader::Loader::load_folders(&mut c, &path).unwrap();
    let s = serde_json::to_string_pretty(&c).unwrap();
    std::fs::write(PathBuf::from("/tmp/temp.json"), s).unwrap();
}
