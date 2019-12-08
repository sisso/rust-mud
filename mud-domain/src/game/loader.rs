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
use crate::game::crafts::Craft;

#[derive(Deserialize, Debug)]
pub struct RoomExitData {
    pub dir: String,
    pub to: StaticId,
}

#[derive(Deserialize, Debug)]
pub struct RoomData {
    pub can_exit: Option<bool>,
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
pub struct CraftData {

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
    /// Is instantiate in same context of parent, ID is mapped
    pub parent: Option<StaticId>,
    /// Are instantiate in own context, unique ID and place as children
    pub children: Option<Vec<StaticId>>,
    pub craft: Option<CraftData>,
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
        debug!("{:?} adding prefab {:?}", id, data);
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

    pub fn find_deep_prefabs_by_parents(&self, static_id: StaticId) -> Vec<StaticId> {
        let mut result = vec![];
        let mut queue = vec![];

        queue.push(static_id);

        while !queue.is_empty() {
            let current = queue.pop().unwrap();
            for child_id in self.find_prefabs_by_parent(current) {
                if result.contains(&child_id) {
                    panic!("recursive reference found on {:?} when searching for {:?}", child_id, static_id);
                }

                result.push(child_id);
                queue.push(child_id);
            }
        }

        result
    }

    pub fn spawn_at(container: &mut Container, static_id: StaticId, parent_id: ObjId) -> SResult<ObjId> {
        let obj_id = Loader::spawn(container, static_id)?;
        container.locations.set(obj_id, parent_id);
        Ok(obj_id)
    }

    pub fn spawn(container: &mut Container, static_id: StaticId) -> SResult<ObjId> {
        debug!("spawn {:?}", static_id);

        let mut references = HashMap::new();

        // create objects
        let obj_id = container.objects.create();
        trace!("spawn {:?} with id {:?}", static_id, obj_id);
        references.insert(static_id, obj_id);

        let children_prefabs = container.loader.find_deep_prefabs_by_parents(static_id);
        for child_static_id in children_prefabs {
            let child_id = container.objects.create();
            trace!("spawn {:?} child {:?} with id {:?}", static_id, child_static_id, child_id);
            references.insert(child_static_id, child_id);
        }

        // initialize all
        for (&static_id, &obj_id) in &references {
            Loader::apply_prefab(container, obj_id, Either::Right(static_id), &references).unwrap();
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
    fn apply_prefab(container: &mut Container, obj_id: ObjId, data: Either<&ObjData, StaticId>, references: &HashMap<StaticId, ObjId>) -> SResult<()> {
        let data: &ObjData =
            match data {
                Either::Left(data) => data,
                Either::Right(static_id) => {
                    container.loader.get_prefab(static_id)
                        .ok_or(format!("prefab {:?} not found", static_id))?
                },
            };

        debug!("{:?} apply prefab {:?}", obj_id, data.id);

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

        if let Some(craft) = &data.craft {
            container.crafts.add(Craft::new(obj_id));
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
            room.can_exit = room_data.can_exit.unwrap_or(false);

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
                trace!("{:?} spawn children {:?}", obj_id, static_id);
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
        let mut ids = HashSet::new();

        for (_static_id, data) in data.objects.iter() {
            if !ids.insert(data.id) {
                return Err(format!("duplicate object id {}", data.id));
            }
        }

        for (_static_id, data) in data.prefabs.iter() {
           if !ids.insert(data.id) {
               return Err(format!("duplicate prefab id {}", data.id));
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
            Loader::apply_prefab(container, ObjId(id.as_u32()), Either::Left(data), &mut empty_references)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::comm::item_body_appears_in_room;

    #[test]
    pub fn intialize_with_spawn() {
        let buffer = r#"
objects.sector_1_dune_palace {
    id: 0,
    label: "Palace"
    desc: "The greate Palace of Dune"
    room: {
      exits: [
        {dir: "s", to: ${objects.sector_1_dune_landing_pad.id} }
      ]
    }
    parent: ${objects.sector_1_dune.id}
}

objects.sector_1_dune_landing_pad {
    id: 1,
    label: "Landing pad"
    desc: "City landing pad."
    room: {
      landing_pad: true
      exits: [
        {dir: "n", to: ${objects.sector_1_dune_palace.id} }
      ]
    }
    parent: ${objects.sector_1_dune.id}
    children: [2]
}

prefabs.control_panel {
    id: 2,
    label: "Control Panel",
}

prefabs.control_panel_command_1 {
    id: 3,
    label: "Command 1",
    parent: 2,
    room: {
        exits: [ {dir: "s", to: 4  } ]
    }
}

prefabs.control_panel_command_2 {
    id: 4,
    label: "Command 2",
    parent: 2,
    room: {
        exits: [ {dir: "n", to: 3  } ]
    }
}
        "#;

        let mut container = Container::new();
        Loader::load_str(&mut container, buffer);

        let landing_pad_id = ObjId(1);

        let landing_pad = container.rooms.get(landing_pad_id).unwrap();
        assert_eq!(ObjId(0), landing_pad.exits.first().unwrap().1);

        let at_landing_pad = container.locations.list_at(landing_pad_id).collect::<Vec<_>>();
        assert_eq!(1, at_landing_pad.len());

        let control_panel_id = *at_landing_pad.first().unwrap();
        let panel_str = container.labels.get_label_f(control_panel_id);
        assert_eq!("Control Panel", panel_str);

        let at_control_panel = container.locations.list_at(control_panel_id).collect::<Vec<_>>();
        assert_eq!(2, at_control_panel.len());

        let mut command1_id = None;
        let mut command2_id = None;

        for id in at_control_panel {
            let label = container.labels.get_label(id).unwrap();
            match label {
                "Command 1" => command1_id = Some(id),
                "Command 2" => command2_id = Some(id),
                other => panic!("Unexpected {:?}", other),
            }
        }

        assert!(command1_id.is_some());
        assert!(command2_id.is_some());

        let room = container.rooms.get(command1_id.unwrap()).unwrap();
        assert_eq!(command2_id.unwrap(), room.exits.first().unwrap().1);

        let room = container.rooms.get(command2_id.unwrap()).unwrap();
        assert_eq!(command1_id.unwrap(), room.exits.first().unwrap().1);
    }
}
