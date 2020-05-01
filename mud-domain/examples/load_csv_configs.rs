use mud_domain::game::container::Container;
use mud_domain::game::loader::Loader;
use std::path::Path;

fn main() {
    let file_path = std::env::args().nth(1).unwrap();
    read(file_path.as_str());
}

fn read(file_path: &str) {
    let content = std::fs::read_to_string(Path::new(file_path)).unwrap();
    let mut container = Container::new();
    Loader::load_from_csv(&mut container, content.as_str()).unwrap();
    println!("done");
}

#[cfg(test)]
mod test {
    #[test]
    fn test_read_csv() {
        let file_path = "../data/fantasy/items.csv";
        super::read(file_path);
    }
}
