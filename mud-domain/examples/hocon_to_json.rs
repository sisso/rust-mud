use hocon::{Hocon, HoconLoader};
use serde::Deserialize;
use serde_json::{Number, Value};
use std::fs;

fn hocon_to_json(hocon: Hocon) -> Option<Value> {
    match hocon {
        Hocon::Boolean(b) => Some(Value::Bool(b)),
        Hocon::Integer(i) => Some(Value::Number(Number::from(i))),
        Hocon::Real(f) => Some(Value::Number(
            Number::from_f64(f).unwrap_or(Number::from(0)),
        )),
        Hocon::String(s) => Some(Value::String(s)),
        Hocon::Array(vec) => Some(Value::Array(
            vec.into_iter()
                .map(hocon_to_json)
                .filter_map(|i| i)
                .collect(),
        )),
        Hocon::Hash(map) => Some(Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, hocon_to_json(v)))
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect(),
        )),
        Hocon::Null => Some(Value::Null),
        Hocon::BadValue(_) => None,
    }
}

fn main() {
    let base_path = std::env::args().nth(1).unwrap();
    let mut loader = HoconLoader::new();

    for file in fs::read_dir(base_path.as_str()).unwrap() {
        let os_string = file.unwrap().file_name();
        let filename = os_string.to_str().unwrap();
        loader = loader
            .load_file(format!("{}/{}", base_path, filename).as_str())
            .unwrap();
    }

    let doc = loader.hocon().unwrap();
    println!("{}", hocon_to_json(doc).unwrap().to_string());
}
