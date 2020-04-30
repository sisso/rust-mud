use commons::csv::{csv_strings_to_tables, parse_csv, tables_to_json};
use std::path::Path;

fn main() {
    let csv = std::fs::read_to_string(Path::new("data/fantasy/config-01.csv"))
        .expect("fail to open file");

    let tables = csv_strings_to_tables(&parse_csv(csv.as_ref())).unwrap();
    let json = tables_to_json(&tables).unwrap();
    let json_str = serde_json::to_string_pretty(&json).unwrap();
    println!("{}", json_str);
    std::fs::write(Path::new("/tmp/01.json"), json_str.as_str());
}
