use super::super::*;
use crate::game::domain::Dir;
use crate::game::loader::{CfgData, Data, ObjData, StaticId};
use crate::game::obj::Obj;
use hocon::{Error as HError, *};
use logs::*;
use std::collections::HashMap;
use std::fs::ReadDir;
use std::io::Error as IError;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    HoconError { error: HError },
    NotObject,
    IOError { error: IError },
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
            Hocon::Hash(map) => Ok(map.keys().map(|i| i.as_str()).collect()),
            _ => Err(Error::NotObject),
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
                }
                "objects" => {
                    HParser::load_all(value, &mut objects)?;
                }
                "prefabs" => {
                    HParser::load_all(value, &mut prefabs)?;
                }

                key => {
                    // any other key is ignored
                    debug!("ignoring key {:?} in config files", key);
                }
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

        for (_key, value) in map {
            let obj: ObjData = HParser::load_obj(value)?;

            let static_id = obj.id;
            if objects.contains_key(&static_id) {
                panic!("duplicated id {:?}", static_id);
            }

            objects.insert(static_id, obj);
        }

        Ok(())
    }

    pub fn load_from_str(input: &str) -> Result<Data, Error> {
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
        assert!(!data.objects.is_empty());
    }

    #[test]
    fn test_load_config() -> Result<(), Error> {
        let sample = r##"
cfg {
    initial_room: 0
    avatar_mob: 1
    initial_craft: 2
}
        "##;

        let data = HParser::load_from_str(sample).unwrap();
        let cfg = data.cfg.expect("cfg field is not defined");
        assert_eq!(cfg.initial_room.as_u32(), 0);
        assert_eq!(cfg.avatar_mob.as_u32(), 1);
        assert_eq!(cfg.initial_craft.unwrap().as_u32(), 2);

        Ok(())
    }

    #[test]
    pub fn test_load_objects() -> Result<(), Error> {
        let sample = r##"
objects {
  sector_1 {
    id: 0
    label: "Sector 1"
    code: ["sector1"]
    sector: {}
  }

  dune: {
    id: 1
    label: "Dune"
    planet: {}
    pos: { x: 3, y: 4 }
    parent: ${objects.sector_1.id}
  }

  palace: {
    id: 2
    label: "Palace"
    desc: "The greate Palace of Dune"
    room: {
      exits: [
        {dir: "s", to: ${objects.landing_pad.id} }
      ]
    }
    parent: ${objects.dune.id}
  }

  landing_pad: {
    id: 3
    label: "Landing pad"
    desc: "City landing pad."
    room: {
      landing_pad: true
      exits: [
        {dir: "n", to: ${objects.palace.id} }
        {dir: "s", to: ${objects.city.id} }
      ]
    }
    parent: ${objects.dune.id}
    children: [1001, 1002]
  }

  city: {
    id: 4
    label: "City center"
    desc: "The deserts market and city center"
    room: {
      exits: [
        {dir: "n", to: ${objects.landing_pad.id} }
      ]
    }
    parent: ${objects.dune.id}
  }
}
        "##;

        let data = HParser::load_from_str(sample).unwrap();
        assert!(data.prefabs.is_empty());
        assert_eq!(5, data.objects.len());

        let (_id, obj_3) = data
            .objects
            .iter()
            .find(|(id, _data)| id.as_u32() == 3)
            .unwrap();

        let obj_3_children = obj_3.children.clone().unwrap();
        assert_eq!(obj_3_children, vec![StaticId(1001), StaticId(1002)]);

        Ok(())
    }

    #[test]
    pub fn test_load_prefabs() -> Result<(), Error> {
        let sample = r##"
prefabs {
  shuttle: {
    id: 0
    label: "Shuttle"
    desc: "Small shuttle"
  }

  shuttle_cockpit: {
    id: 1
    label: "Cockpit"
    desc: "Small cockpit used to control the craft"
    room {
      airlock: true
    }
    parent: 0
  }
}
          "##;

        let data = HParser::load_from_str(sample).unwrap();
        assert!(data.objects.is_empty());
        assert_eq!(2, data.prefabs.len());
        Ok(())
    }
}
