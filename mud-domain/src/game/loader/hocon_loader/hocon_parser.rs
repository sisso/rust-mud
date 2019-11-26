use std::fs::ReadDir;
use std::path::Path;
use logs::*;
use hocon::{*, Error as HError};
use crate::game::domain::Dir;
use std::collections::HashMap;
use std::io::Error as IError;
use crate::game::obj::Obj;
use super::super::*;

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


pub struct HParser;

impl HParser {
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
                    cfg = Some(HParser::load_cfg(value)?);
                },
                "objects" => {
                    HParser::load_all(value, &mut objects)?;
                },
                "prefabs" => {
                    HParser::load_all(value, &mut prefabs)?;
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
            let obj: ObjData = HParser::load_obj(value)?;
            objects.insert(StaticId(key), obj);
        }

        Ok(())
    }

    fn load_from_str(input: &str) -> Result<Data, Error> {
        let loader = HoconLoader::new().no_system();
        let loader = loader.load_str(input)?;
        HParser::load(loader.hocon()?)
    }

    pub fn load_from_folder(path: &Path) -> Result<Data, Error> {
        let mut loader = HoconLoader::new().no_system();
        let list: ReadDir = std::fs::read_dir(path)?;
        for entry in list {
            let path = entry?.path();
            info!("loading configuration file {:?}", path);
            loader = loader.load_file(path.to_str().unwrap())?;
        }

        HParser::load(loader.hocon()?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_load_folder() {
        let path = Path::new("../data/space");

        let data = HParser::load_from_folder(path).unwrap();
        println!("{:?}", data);
        assert!(!data.objects.is_empty());
        assert!(false);
    }

    #[test]
    fn test_load_config() -> Result<(), Error> {
        let sample = r##"
cfg {
    initial_room: "sector_1/dune/palace"
    avatar_mob: "avatar"
    initial_craft: "light_cargo_1"
}
        "##;

        let data= HParser::load_from_str(sample).unwrap();
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

        let data= HParser::load_from_str(sample).unwrap();
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

        let data= HParser::load_from_str(sample).unwrap();
        assert!(data.objects.is_empty());
        assert_eq!(2, data.prefabs.len());
        Ok(())
    }
}
