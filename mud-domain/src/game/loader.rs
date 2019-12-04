pub mod scenery_fantasy;
pub mod scenery_space;
mod hocon_parser;

use serde::Deserialize;
use std::collections::HashMap;
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
use commons::{ObjId, V2, SResult};
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

    pub fn as_id(&self) -> ObjId {
        ObjId(self.0)
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

    pub fn spawn(container: &mut Container, id: StaticId) -> Result<ObjId, String> {
        let obj_id = container.objects.create();
        Loader::do_spawn(container, id, obj_id, HashMap::new())
    }

    /// Spawn a singleton of prefab, it can only be one and ObjId == StaticId
    fn initialize(container: &mut Container, id: StaticId) -> Result<ObjId, String> {
        let obj_id = ObjId(id.as_u32());
        // TODO: move to result
        container.objects.insert(obj_id);
        Loader::do_spawn(container, id, obj_id, HashMap::new())
    }

    fn get_by_static_id(objects: &Objects, id_map: &HashMap<StaticId, ObjId>, static_id: StaticId) -> SResult<ObjId> {
        // search from map and fallback to real ObjId
        id_map.get(&static_id)
            .cloned()
            .or_else(|| {
                let id = static_id.as_id();
                if objects.exists(id) {
                    Some(id)
                } else {
                    None
                }
            }).ok_or_else(|| format!("obj id for static_id {:?} not found", static_id))
    }

    // TODO: make it atomic: success and change or no change
    fn do_spawn(container: &mut Container, id: StaticId, obj_id: ObjId, id_map: HashMap<StaticId, ObjId>) -> SResult<ObjId> {
        let data = container.loader.get_prefab(id)
            .ok_or(format!("prefab {:?} not found", id))?;

        if let Some(parent) = &data.parent {
            let parent_id = Loader::get_by_static_id(&container.objects, &id_map, *parent)?;
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
                    let to_id = Loader::get_by_static_id(&container.objects, &id_map, i.to)?;

                    room.exits.push((dir, to_id));
                }
            }

            container.rooms.add(room);
        }

        Ok(obj_id)
    }

    pub fn load_folder(container: &mut Container, folder: &Path) -> SResult<()> {
        if !folder.exists() {
            return Err("module folder do not exists".to_string());
        }

        let data = HParser::load_from_folder(folder)
            .map_err(|e| format!("{:?}", e))?;

        // update configurations with references
        if let Some(cfg) = data.cfg {
            container.config.initial_room = cfg.initial_room.as_id();
        }

        // collect objects that need to be initialized on load
        let initialized = data.objects.keys().cloned().collect::<Vec<StaticId>>();

        // add prefabs
        for (k, v) in data.prefabs {
            container.loader.add_prefab(k, v);
        }

        for (k, v) in data.objects{
            container.loader.add_prefab(k, v);
        }

        // initialize objects
        for static_id in initialized {
            Loader::initialize(container, static_id);
        }

        Ok(())
    }

    fn spawn_all(
        container: &mut Container,
        objects: HashMap<StaticId, ObjData>,
    ) -> HashMap<StaticId, ObjId> {
        // create object by id
        let mut obj_by_id = HashMap::new();

        for (key, _) in &objects {
            let obj_id = container.objects.create();
            obj_by_id.insert(key.clone(), obj_id);
        }

        for (key, data) in objects.into_iter() {
            let obj_id = *obj_by_id.get(&key).unwrap();

            {
                let label = data.label;
                let code = data
                    .code
                    .map(|i| i.first().cloned())
                    .and_then(|o| o)
                    .unwrap_or(label.clone());
                let desc = data.desc.unwrap_or("".to_string());

                container.labels.set(Label {
                    id: obj_id,
                    label,
                    code,
                    desc,
                });
            }

            if let Some(pos) = data.pos {
                container.pos.set(obj_id, V2::new(pos.x, pos.y));
            }

            if let Some(_planet) = data.planet {
                container.planets.add(Planet { id: obj_id });
            }

            if let Some(_surfaces) = data.sector {
                container.sectors.add(Surface {
                    id: obj_id,
                    size: 10,
                    is_3d: false,
                });
            }

            if let Some(mob_data) = data.mob {
                let mut mob = Mob::new(obj_id);
                mob.attributes.attack = mob_data.attack;
                mob.attributes.defense = mob_data.defense;
                mob.attributes.pv.current = mob_data.pv as i32;
                mob.attributes.pv.max = mob_data.pv;
                mob.attributes.damage.max = mob_data.damage_max;
                mob.attributes.damage.min = mob_data.damage_min;
                container.mobs.add(mob);
            }

            if let Some(room_data) = data.room {
                let mut room = Room::new(obj_id);
                room.is_airlock = room_data.airlock.unwrap_or(false);
                let exits = room_data
                    .exits
                    .unwrap_or(vec![])
                    .into_iter()
                    .map(|e| {
                        let dir = Dir::parse(e.dir.as_str()).unwrap();
                        let to_id = obj_by_id.get(&e.to).unwrap();

                        (dir, *to_id)
                    })
                    .collect();

                room.exits = exits;

                container.rooms.add(room);
            }

            if let Some(parent) = data.parent {
                let parent_id = obj_by_id.get(&parent).unwrap();
                container.locations.set(obj_id, *parent_id)
            }
        }

        obj_by_id
    }
}
