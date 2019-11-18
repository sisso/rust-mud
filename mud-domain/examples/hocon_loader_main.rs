use std::fs;
use hocon::{Hocon, HoconLoader, Error};
use serde_json::{Number, Value};
use std::collections::HashMap;
use mud_domain::game::domain::Dir;

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
            Hocon::Hash(map) => {
                Ok(map.keys().map(|i| i.as_str()).collect())
            },
            _ => Err(Error::InvalidKey)
        }
    }
}

pub struct RoomExitData {
    pub dir: Dir,
    pub to: StaticId,
}

pub struct RoomData {
    pub airlock: bool,
    pub exits: Vec<RoomExitData>
}

pub struct PlanetData {

}

pub struct SectorData {

}

pub struct MobData {
    attack: u32,
    defense: u32,
    damage_min: u32,
    damage_max: u32,
    pv: u32
}

pub struct PosData {
    pub x: f32,
    pub y: f32,
}

pub struct StaticId(pub String);

pub struct ObjData {
    pub id: StaticId,
    pub label: String,
    pub code: Vec<String>,
    pub desc: String,
    pub room: Option<RoomData>,
    pub planet: Option<PlanetData>,
    pub sector: Option<SectorData>,
    pub mob: Option<MobData>,
    pub pos: Option<PosData>,
}

pub struct CfgData {
    initial_room: StaticId,
    avatar_mob: StaticId,
    initial_craft: StaticId,
}

pub struct Data {
    pub cfg: CfgData,
    pub objects: HashMap<StaticId, ObjData>,
    pub prefabs: HashMap<StaticId, ObjData>,
}

fn parse(hocon: Hocon) -> Result<(), Error> {
    for id in hocon.keys()? {
        println!("{:?}", id);
        let value = hocon[id].keys().expect("All object must have keys");
        println!("{:?}", value);

        // parse special keys
        if id == "cfg" {
            println!("configuration");
        } else {
            println!("object");
        }
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let base_path = "./data/space";
    let mut loader = HoconLoader::new();

    for file in fs::read_dir(base_path).unwrap() {
        let os_string = file.unwrap().file_name();
        let filename = os_string.to_str().unwrap();
        let mut loader = loader.load_file(format!("{}/{}", base_path, filename).as_str())?;
        let doc = loader.hocon()?;
        println!("----------------------\n{}\n------------------", filename);
//        println!("{}", serde_json::to_string_pretty(&hocon_to_json(doc)).unwrap());
        parse(doc)?;
    }

    Ok(())
}
