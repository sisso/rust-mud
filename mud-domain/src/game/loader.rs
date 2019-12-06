pub mod scenery_fantasy;
pub mod scenery_space;
mod hocon_parser;

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use crate::game::container::Container;
use crate::game::domain::Dir;
use crate::game::labels::Label;
use crate::game::loader::hocon_parser::HParser;
use crate::game::mob::Mob;
use crate::game::planets::Planet;
use crate::game::pos::Pos;
use crate::game::room::Room;
use crate::game::surfaces::Surface;
use commons::{ObjId, V2, SResult, Either};
use logs::*;
use crate::game::obj::Objects;

#[derive(Deserialize, Debug)]
pub struct RoomExitData {
    pub dir: String,
    pub to: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct RoomData {
    pub airlock: Option<bool>,
    pub exits: Option<Vec<RoomExitData>>,
}

#[derive(Deserialize, Debug)]
pub struct PlanetData {}

#[derive(Deserialize, Debug)]
pub struct SectorData {}

#[derive(Deserialize, Debug)]
pub struct MobData {
    pub attack: u32,
    pub defense: u32,
    pub damage_min: u32,
    pub damage_max: u32,
    pub pv: u32,
}

#[derive(Deserialize, Debug)]
pub struct PosData {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Hash, Eq, Copy)]
pub struct StaticId(pub u32);

impl StaticId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Deserialize, Debug)]
pub struct ObjData {
    pub id: u32,
    pub label: String,
    pub code: Option<Vec<String>>,
    pub desc: Option<String>,
    pub room: Option<RoomData>,
    pub planet: Option<PlanetData>,
    pub sector: Option<SectorData>,
    pub mob: Option<MobData>,
    pub pos: Option<PosData>,
    pub parent: Option<StaticId>,
    pub children: Option<Vec<StaticId>>
}

#[derive(Deserialize, Debug)]
pub struct CfgData {
    pub initial_room: StaticId,
    pub avatar_mob: StaticId,
    pub initial_craft: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub cfg: Option<CfgData>,
    pub objects: HashMap<StaticId, ObjData>,
    pub prefabs: HashMap<StaticId, ObjData>,
}

impl Data {
    pub fn new() -> Self {
        Data {
            cfg: None, objects: Default::default(), prefabs: Default::default()
        }
    }
}

pub struct Loader {
    index: HashMap<StaticId, ObjData>,
}

impl Loader {
    pub fn new() -> Self {
        Loader {
            index: Default::default(),
        }
    }

    pub fn add_prefab(&mut self, id: StaticId, data: ObjData) {
        assert!(!self.index.contains_key(&id));
        info!("{:?} adding prefab {:?}", id, data);
        self.index.insert(id, data);
    }

    pub fn get_prefab(&self, id: StaticId) -> Option<&ObjData> {
        self.index.get(&id)
    }

    pub fn find_prefabs_by_parent(&self, id: StaticId) -> Vec<StaticId> {
        self.index.iter()
            .filter(|(_, data)| {
                data.parent
                    .map(|parent_id| parent_id == id)
                    .unwrap_or(false)
            })
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn spawn_at(container: &mut Container, static_id: StaticId, parent_id: ObjId) -> SResult<ObjId> {
        let obj_id = Loader::spawn(container, static_id)?;
        container.locations.set(obj_id, parent_id);
        Ok(obj_id)
    }
    pub fn spawn(container: &mut Container, id: StaticId) -> SResult<ObjId> {
        let mut references = HashMap::new();
        Loader::spawn_one(container, id, &mut references)
    }

    fn spawn_one(container: &mut Container, id: StaticId, references: &mut HashMap<StaticId, ObjId>) -> SResult<ObjId> {
        let obj_id = container.objects.create();
        references.insert(id, obj_id);
        Loader::do_spawn(container, obj_id, Either::Right(id), &references)?;

        let children_prefabs = container.loader.find_prefabs_by_parent(id);
        for children_static_id in children_prefabs {
            Loader::spawn_one(container, children_static_id, references)?;
        }

        Ok(obj_id)
    }

    /// Resolve the static id to a ObjId by first searching in reference_map and then in container
    fn get_by_static_id(objects: &Objects, ref_map: &HashMap<StaticId, ObjId>, static_id: StaticId) -> SResult<ObjId> {
        // search from map and fallback to real ObjId
        ref_map.get(&static_id)
            .cloned()
            .or_else(|| {
                let id = ObjId(static_id.as_u32());
                if objects.exists(id) {
                    Some(id)
                } else {
                    None
                }
            }).ok_or_else(|| format!("obj id for static_id {:?} not found", static_id))
    }

    // TODO: make it atomic: success and change or no change
    fn do_spawn(container: &mut Container, obj_id: ObjId, data: Either<&ObjData, StaticId>, references: &HashMap<StaticId, ObjId>) -> SResult<()> {
        let data: &ObjData =
            match data {
                Either::Left(data) => data,
                Either::Right(static_id) => {
                    container.loader.get_prefab(static_id)
                        .ok_or(format!("prefab {:?} not found", static_id))?
                },
            };

        info!("{:?} spawn as {:?}", obj_id, data.id);

        if let Some(parent) = &data.parent {
            let parent_id = Loader::get_by_static_id(&container.objects, &references, *parent)?;
            container.locations.set(obj_id, parent_id)
        }

        {
            let label = data.label.clone();
            // TODO: simplify
            let code = data
                .code
                .clone()
                .map(|i| i.first().cloned())
                .and_then(|o| o)
                .unwrap_or(label.clone());
            let desc = data.desc.clone().unwrap_or("".to_string());

            container.labels.set(Label {
                id: obj_id,
                label,
                code,
                desc,
            });
        }

        if let Some(pos) = &data.pos {
            container.pos.set(obj_id, V2::new(pos.x, pos.y));
        }

        if let Some(_planet) = &data.planet {
            container.planets.add(Planet { id: obj_id });
        }

        if let Some(_surfaces) = &data.sector {
            container.sectors.add(Surface {
                id: obj_id,
                size: 10,
                is_3d: false,
            });
        }

        if let Some(mob_data) = &data.mob {
            let mut mob = Mob::new(obj_id);
            mob.attributes.attack = mob_data.attack;
            mob.attributes.defense = mob_data.defense;
            mob.attributes.pv.current = mob_data.pv as i32;
            mob.attributes.pv.max = mob_data.pv;
            mob.attributes.damage.max = mob_data.damage_max;
            mob.attributes.damage.min = mob_data.damage_min;
            container.mobs.add(mob);
        }

        if let Some(room_data) = &data.room {
            let mut room = Room::new(obj_id);
            room.is_airlock = room_data.airlock.unwrap_or(false);

            if let Some(exists) = &room_data.exits {
                for i in exists {
                    let dir = Dir::parse(i.dir.as_str())
                        .map_err(|e| format!("{:?}", e))?;
                    let to_id = Loader::get_by_static_id(&container.objects, &references, i.to).unwrap();

                    room.exits.push((dir, to_id));
                }
            }

            container.rooms.add(room);
        }

        if let Some(children) = data.children.clone() {
            for static_id in children.into_iter() {
                info!("{:?} spawn children {:?}", obj_id, static_id);
                Loader::spawn_at(container, static_id, obj_id)?;
            }
        }

        Ok(())
    }

    pub fn load_str(container: &mut Container, buffer: &str) -> SResult<()> {
       let data = HParser::load_from_str(buffer)
           .map_err(|e| format!("{:?}", e))?;

        Loader::load_data(container, data)
    }


    /// Algorithm
    ///
    /// 1. Load all files and resolve variables
    /// 2. Validate content
    /// 3. Add all prefabs
    /// 4. Instantiate all static data
    pub fn load_folder(container: &mut Container, folder: &Path) -> SResult<()> {
        if !folder.exists() {
            return Err("module folder do not exists".to_string());
        }

        let data = HParser::load_from_folder(folder)
            .map_err(|e| format!("{:?}", e))?;

        Loader::load_data(container, data)
    }

    fn load_data(container: &mut Container, data: Data) -> SResult<()> {
        let _ = Loader::validate(&data)?;

        // add prefabs
        for (k, v) in data.prefabs {
            container.loader.add_prefab(k, v);
        }

        // instantiate static data
        Loader::initialize_all(container, data.objects);

        // update configurations with references
        if let Some(cfg) = data.cfg {
            container.config.initial_room = ObjId(cfg.initial_room.as_u32());
        }

        Ok(())
    }

    fn validate(data: &Data) -> SResult<()> {
        let mut prefabs_id = HashSet::new();
        for (_static_id, data) in data.prefabs.iter() {
           if !prefabs_id.insert(data.id) {
               return Err(format!("duplicate prefab id {}", data.id));
           }
        }

        let mut objects_id = HashSet::new();
        for (_static_id, data) in data.prefabs.iter() {
            if !objects_id.insert(data.id) {
                return Err(format!("duplicate object id {}", data.id));
            }
        }

        Ok(())
    }

    fn initialize_all(
        container: &mut Container,
        objects: HashMap<StaticId, ObjData>,
    ) -> SResult<()> {
        for (key, _) in &objects {
            // TODO: to result
            container.objects.insert(ObjId(key.as_u32()));
        }

        for (id, data) in &objects {
            let mut empty_references = Default::default();
            Loader::do_spawn(container, ObjId(id.as_u32()), Either::Left(data), &mut empty_references)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn intializer_with_spawn() {
        let mut container = Container::new();
        unimplemented!()
    }
}
