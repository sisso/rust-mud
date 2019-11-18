use std::fs::ReadDir;
use std::path::Path;
use logs::*;
use hocon::*;
use crate::game::domain::Dir;
use std::collections::HashMap;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

trait HoconExtra {
    fn keys(&self) -> Result<Vec<&str>>;
}

impl HoconExtra for Hocon {
    fn keys(&self) -> Result<Vec<&str>> {
        match self {
            Hocon::Hash(map) => {
                Ok(map.keys().map(|i| i.as_str()).collect())
            },
            _ => Err(Box::new(Error::InvalidKey))
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
pub struct HLoader;

impl HLoader {
    pub fn load(hocon: Hocon) -> Result<Data> {
        unimplemented!()
    }

    fn load_from_str(input: &str) -> Result<Data> {
        let loader = HoconLoader::new();
        let loader = loader.load_str(input)?;
        HLoader::load(loader.hocon()?)
    }

    fn load_from_folder(path: &Path) -> Result<Data> {
        let mut loader = HoconLoader::new();
        let list: ReadDir = std::fs::read_dir(path)?;
        for entry in list {
            let path = entry?.path();
            info!("loading configuration file {:?}", path);
            loader = loader.load_file(path.to_str().unwrap())?;
        }

        let doc = loader.hocon()?;
        HLoader::load(doc)
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_load_config() {

    }
}
