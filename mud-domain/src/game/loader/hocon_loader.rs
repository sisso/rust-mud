use serde::Deserialize;
use std::fs::ReadDir;
use std::path::Path;
use logs::*;
use hocon::{*, Error as HError};
use crate::game::domain::Dir;
use std::collections::HashMap;
use crate::game::loader::hocon_loader::Error::HoconError;
use std::io::Error as IError;
use crate::game::obj::Obj;

#[derive(Debug)]
pub enum Error {
    HoconError { error: HError },
    NotObject,
    IOError { error: IError }
}

impl From<HError> for Error {
    fn from(error: HError) -> Self {
        Error::HoconError { error }
    }
}

impl From<IError> for Error {
    fn from(error: IError) -> Self {
        Error::IOError { error }
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
            _ => Err(Error::NotObject)
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RoomExitData {
    pub dir: String,
    pub to: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct RoomData {
    pub airlock: Option<bool>,
    pub exits: Option<Vec<RoomExitData>>
}

#[derive(Deserialize, Debug)]
pub struct PlanetData {

}

#[derive(Deserialize, Debug)]
pub struct SectorData {

}

#[derive(Deserialize, Debug)]
pub struct MobData {
    attack: u32,
    defense: u32,
    damage_min: u32,
    damage_max: u32,
    pv: u32
}

#[derive(Deserialize, Debug)]
pub struct PosData {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct StaticId(pub String);

impl StaticId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Deserialize, Debug)]
pub struct ObjData {
    pub label: String,
    pub code: Option<Vec<String>>,
    pub desc: Option<String>,
    pub room: Option<RoomData>,
    pub planet: Option<PlanetData>,
    pub sector: Option<SectorData>,
    pub mob: Option<MobData>,
    pub pos: Option<PosData>,
    pub parent: Option<StaticId>,
}

#[derive(Deserialize, Debug)]
pub struct CfgData {
    initial_room: StaticId,
    avatar_mob: StaticId,
    initial_craft: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub cfg: Option<CfgData>,
    pub objects: HashMap<StaticId, ObjData>,
    pub prefabs: HashMap<StaticId, ObjData>,
}

pub struct HLoader;

impl HLoader {
    pub fn load(hocon: Hocon) -> Result<Data, Error> {
        let mut cfg = None;
        let mut objects = HashMap::new();
        let mut prefabs = HashMap::new();

        let map = match hocon {
            Hocon::Hash(map) => map,
            _ => return Err(Error::NotObject),
        };

        for (key, value) in map {
            match key.as_str() {
                "cfg" => {
                    cfg = Some(HLoader::load_cfg(value)?);
                },
                "objects" => {
                    HLoader::load_all(value, &mut objects)?;
                },
                "prefabs" => {
                    HLoader::load_all(value, &mut prefabs)?;
                },

                _ => unimplemented!(),
            }
        }

        Ok(Data {
            cfg,
            objects,
            prefabs,
        })
    }

    fn load_cfg(hocon: Hocon) -> Result<CfgData, Error> {
        hocon.resolve().map_err(|e| e.into())
    }

    fn load_obj(hocon: Hocon) -> Result<ObjData, Error> {
        hocon.resolve().map_err(|e| e.into())
    }

    fn load_all(hocon: Hocon, objects: &mut HashMap<StaticId, ObjData>) -> Result<(), Error> {
        let map = match hocon {
            Hocon::Hash(map) => map,
            _ => return Err(Error::NotObject),
        };

        for (key, value) in map {
            let obj: ObjData = HLoader::load_obj(value)?;
            objects.insert(StaticId(key), obj);
        }

        Ok(())
    }

    fn load_from_str(input: &str) -> Result<Data, Error> {
        let loader = HoconLoader::new().no_system();
        let loader = loader.load_str(input)?;
        HLoader::load(loader.hocon()?)
    }

    fn load_from_folder(path: &Path) -> Result<Data, Error> {
        let mut loader = HoconLoader::new().no_system();
        let list: ReadDir = std::fs::read_dir(path)?;
        for entry in list {
            let path = entry?.path();
            info!("loading configuration file {:?}", path);
            loader = loader.load_file(path.to_str().unwrap())?;
        }

        HLoader::load(loader.hocon()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_load_config() -> Result<(), Error>{
        let sample = r##"
cfg {
    initial_room: "sector_1/dune/palace"
    avatar_mob: "avatar"
    initial_craft: "light_cargo_1"
}
        "##;

        let data= HLoader::load_from_str(sample).unwrap();
        let cfg = data.cfg.expect("cfg field is not defined");
        assert_eq!(cfg.initial_room.as_str(), "sector_1/dune/palace");
        assert_eq!(cfg.avatar_mob.as_str(), "avatar");
        assert_eq!(cfg.initial_craft.as_str(), "light_cargo_1");

        Ok(())
    }

    #[test]
    pub fn test_load_objects() -> Result<(), Error>{
        let sample = r##"
objects {
  sector_1 {
    label: "Sector 1"
    code: ["sector1"]
    sector: {}
  }

  dune: {
    label: "Dune"
    planet: {}
    pos: { x: 3, y: 4 }
    parent: "sector_1"
  }

  palace: {
    label: "Palace"
    desc: "The greate Palace of Dune"
    room: {
      exits: [
        {dir: "s", to: "landing_pad"}
      ]
    }
    parent: "dune"
  }

  landing_pad: {
    label: "Landing pad"
    desc: "City landing pad."
    room: {
      landing_pad: true
      exits: [
        {dir: "n", to: "palace"}
        {dir: "s", to: "city"}
      ]
    }
    parent: "dune"
  }

  city: {
    label: "City center"
    desc: "The deserts market and city center"
    room: {
      exits: [
        {dir: "n", to: "landing_pad"}
      ]
    }
    parent: "dune"
  }
}
        "##;

        let data= HLoader::load_from_str(sample).unwrap();
        assert!(data.prefabs.is_empty());
        assert_eq!(5, data.objects.len());
        Ok(())
    }

    #[test]
    pub fn test_load_prefabs() -> Result<(), Error>{
        let sample = r##"
prefabs {
  shuttle: {
    label: "Shuttle"
    desc: "Small shuttle"
  }

  shuttle_cockpit: {
    label: "Cockpit"
    desc: "Small cockpit used to control the craft"
    room {
      airlock: true
    }
    parent: "shuttle"
  }
}
          "##;

        let data= HLoader::load_from_str(sample).unwrap();
        assert!(data.objects.is_empty());
        assert_eq!(2, data.prefabs.len());
        Ok(())
    }
}
