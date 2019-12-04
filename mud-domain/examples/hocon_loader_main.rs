use hocon::{Error, Hocon, HoconLoader};
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

trait HoconExtra {
    fn keys(&self) -> Result<Vec<&str>, Error>;
}

impl HoconExtra for Hocon {
    fn keys(&self) -> Result<Vec<&str>, Error> {
        match self {
            Hocon::Hash(map) => Ok(map.keys().map(|i| i.as_str()).collect()),
            _ => Err(Error::InvalidKey),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Data {
    a: u32,
}

fn parse(hocon: Hocon) -> Result<(), Error> {
    println!("{}", hocon_to_json(hocon).unwrap().to_string());
    Ok(())
}

fn main() -> Result<(), Error> {
    let base_path = "./data/space";
    let loader = HoconLoader::new();

    for file in fs::read_dir(base_path).unwrap() {
        let os_string = file.unwrap().file_name();
        let filename = os_string.to_str().unwrap();
        let loader = loader.load_file(format!("{}/{}", base_path, filename).as_str())?;
        let doc = loader.hocon()?;
        println!("----------------------\n{}\n------------------", filename);
        //        println!("{}", serde_json::to_string_pretty(&hocon_to_json(doc)).unwrap());
        parse(doc)?;
    }

    Ok(())
}
