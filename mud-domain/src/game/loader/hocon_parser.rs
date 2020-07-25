use crate::errors::Result as EResult;
use crate::game::domain::Dir;
use crate::game::loader::dto::LoaderData;
use crate::game::loader::{CfgData, ObjData, StaticId};
use crate::game::obj::Obj;
use hocon::{Error as HError, *};
use logs::*;
use std::collections::HashMap;
use std::fs::ReadDir;
use std::io::Error as IError;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ParseError {
    HoconError { error: HError, hint: String },
    NotObject,
    IOError { error: IError },
}

impl From<IError> for ParseError {
    fn from(error: IError) -> Self {
        ParseError::IOError { error }
    }
}

trait HoconExtra {
    fn keys(&self) -> Result<Vec<&str>, ParseError>;
}

impl HoconExtra for Hocon {
    fn keys(&self) -> Result<Vec<&str>, ParseError> {
        match self {
            Hocon::Hash(map) => Ok(map.keys().map(|i| i.as_str()).collect()),
            _ => Err(ParseError::NotObject),
        }
    }
}

pub struct HParser;

impl HParser {
    pub fn parse(data: &mut LoaderData, hocon: Hocon) -> Result<(), ParseError> {
        let mut cfg = None;
        let mut objects = HashMap::new();
        let mut prefabs = HashMap::new();

        let map = match hocon {
            Hocon::Hash(map) => map,
            _ => return Err(ParseError::NotObject),
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

                _key => {
                    // any other key is ignored
                    debug!("ignoring key {:?} in config files", key);
                }
            }
        }

        assert!(data.cfg.is_none());
        data.cfg = cfg;
        data.objects.extend(objects);
        data.prefabs.extend(prefabs);
        Ok(())
    }

    fn load_cfg(hocon: Hocon) -> Result<CfgData, ParseError> {
        hocon
            .clone()
            .resolve()
            .map_err(|error| ParseError::HoconError {
                error,
                hint: format!("hocon '{:?}'", hocon),
            })
    }

    fn load_obj(hocon: Hocon) -> Result<ObjData, ParseError> {
        hocon
            .clone()
            .resolve()
            .map_err(|error| ParseError::HoconError {
                error,
                hint: format!("hocon '{:?}'", hocon),
            })
    }

    fn load_all(hocon: Hocon, objects: &mut HashMap<StaticId, ObjData>) -> Result<(), ParseError> {
        let map = match hocon {
            Hocon::Hash(map) => map,
            _ => return Err(ParseError::NotObject),
        };

        for (_key, value) in map {
            let obj: ObjData = HParser::load_obj(value)?;

            let static_id = obj.id;
            if objects.contains_key(&static_id) {
                warn!("duplicated id {:?}, ignoring", static_id);
            } else {
                objects.insert(static_id, obj);
            }
        }

        Ok(())
    }

    pub fn load_hocon_str(input: &str) -> Result<LoaderData, ParseError> {
        let loader = HoconLoader::new().strict().no_system();
        let loader = loader
            .load_str(input)
            .map_err(|error| ParseError::HoconError {
                error,
                hint: format!("loader for {:?}", input),
            })?;

        let raw = loader.hocon().map_err(|error| ParseError::HoconError {
            error,
            hint: format!("converting to hocon for {:?}", input),
        })?;

        let mut data = LoaderData::new();
        HParser::parse(&mut data, raw)?;
        Ok(data)
    }

    pub fn load_hocon_files<T: AsRef<Path>>(
        data: &mut LoaderData,
        files: &Vec<T>,
    ) -> Result<(), ParseError> {
        let mut loader = HoconLoader::new().strict().no_system();

        for path in files {
            debug!("reading file {:?}", path);

            loader = loader
                .load_file(path)
                .map_err(|error| ParseError::HoconError {
                    error,
                    hint: format!("path '{:?}'", path.as_ref()),
                })?;
        }

        let raw = loader.hocon().map_err(|error| ParseError::HoconError {
            error,
            hint: format!(
                "root_path'{:?}'",
                files.iter().map(|i| i.as_ref()).collect::<Vec<_>>()
            ),
        })?;

        HParser::parse(data, raw)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_config() {
        let sample = r##"cfg {
    initial_room: 0
    avatar_mob: 1
    initial_craft: 2
}"##;

        let data = HParser::load_hocon_str(sample).unwrap();
        let cfg = data.cfg.expect("cfg field is not defined");
        assert_eq!(cfg.initial_room.as_u32(), 0);
        assert_eq!(cfg.avatar_mob.as_u32(), 1);
        assert_eq!(cfg.initial_craft.unwrap().as_u32(), 2);
    }

    #[test]
    pub fn test_load_objects() {
        let sample = r##"objects {
  sector_1 {
    id: 0
    label: "Sector 1"
    code: ["sector1"]
    sector: {}
  }
//
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
}"##;

        let data = HParser::load_hocon_str(sample).unwrap();
        assert!(data.prefabs.is_empty());
        assert_eq!(5, data.objects.len());

        let (_id, obj_3) = data
            .objects
            .iter()
            .find(|(id, _data)| id.as_u32() == 3)
            .unwrap();

        let obj_3_children = obj_3.children.clone().unwrap();
        assert_eq!(obj_3_children, vec![StaticId(1001), StaticId(1002)]);
    }

    #[test]
    pub fn test_load_prefabs() {
        let sample = r##"prefabs {
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
}"##;

        let data = HParser::load_hocon_str(sample).unwrap();
        assert!(data.objects.is_empty());
        assert_eq!(2, data.prefabs.len());
    }
}
