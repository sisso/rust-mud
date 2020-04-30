use commons::csv::{csv_strings_to_tables, parse_csv, tables_to_json, tables_to_jsonp};
use std::collections::HashMap;
use std::path::Path;

fn main() {
    let file_path = std::env::args().nth(1).unwrap();
    let csv = std::fs::read_to_string(Path::new(file_path.as_str())).expect("fail to open file");

    let tables = csv_strings_to_tables(&parse_csv(csv.as_ref())).unwrap();
    let json = tables_to_jsonp(&tables, &HashMap::new()).unwrap();
    let json_str = serde_json::to_string_pretty(&json).unwrap();
    println!("{}", json_str);
    std::fs::write(Path::new("/tmp/01.json"), json_str.as_str());
}
