
use hocon::{*, HoconLoader as HLoader};

use std::path::Path;
use std::fs::ReadDir;
use serde_json::{Number, Value};

use crate::game::loader::{Loader, Result, LoaderError};
use crate::game::container::Container;
use crate::utils::save::Load;

pub struct HoconLoader {

}

impl Loader for HoconLoader {
    fn load(path: &Path) -> Result<Box<dyn Load>> {
        let mut loader = HLoader::new();
        let list: ReadDir = std::fs::read_dir(path)?;
        for entry in list {
            let path = entry?.path();
            eprintln!("parsing {:?}", path);
            let path_str = match path.to_str() {
                Some(str) => str,
                None => {
                    eprintln!("fail to parse no str path");
                    continue;
                }
            };

            loader = loader.load_file(path_str)?;
        }

        let doc = loader.hocon()?;

        match doc {
            Hocon::Hash(map) => {
                let items = map.get("items");
                let rooms = map.get("rooms");
                let mobs = map.get("mobs");
                let tags = map.get("tags");
                let spawns = map.get("spawns");
            },
            _ => return Err(LoaderError::Unknown.into())
        }

        unimplemented!()
    }
}

struct HoconLoad {

}

impl Load for HoconLoad {
    fn get_headers(&mut self, header: &str) -> Vec<&Value> {
        unimplemented!()
    }

    fn get_components(&mut self, component: &str) -> Vec<&(u32, Value)> {
        unimplemented!()
    }
}

/// copied from hocon library examples
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
