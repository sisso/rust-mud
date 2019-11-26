use crate::game::container::Container;
use std::path::Path;
use std::collections::HashMap;
use crate::game::loader::hocon_loader::hocon_parser::{HParser};
use crate::game::labels::Label;
use crate::game::pos::Pos;
use commons::{V2, ObjId};
use crate::game::surfaces::Surface;
use crate::game::planets::Planet;
use crate::game::mob::Mob;
use crate::game::room::Room;
use crate::game::domain::Dir;
use super::*;

mod hocon_parser;

pub fn load(module_folder: &Path, container: &mut Container) {
    assert!(module_folder.exists());

    let data = HParser::load_from_folder(module_folder).unwrap();

    // convert update configuration
    // convert prefabs into builder
    let id_by_static_id= load_objects(container, data.objects);

    // update configurations with references
    if let Some(cfg) = data.cfg {
        container.config.initial_room = *id_by_static_id.get(&cfg.initial_room).unwrap();
    }

    // create prefabs

}

fn load_objects(container: &mut Container, objects: HashMap<StaticId, ObjData>) -> HashMap<StaticId, ObjId> {
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
            let code = data.code.map(|i| i.first().cloned())
                .and_then(|o| o)
                .unwrap_or(label.clone());
            let desc = data.desc.unwrap_or("".to_string());

            container.labels.set(Label {
                id: obj_id,
                label,
                code,
                desc
            });
        }

        if let Some(pos) = data.pos {
            container.pos.set(obj_id, V2::new(pos.x, pos.y));
        }

        if let Some(planet) = data.planet {
            container.planets.add(Planet {
               id: obj_id
            });
        }

        if let Some(surfaces) = data.sector {
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
            let mut exits = room_data.exits.unwrap_or(vec![]).into_iter().map(|e| {
                let dir = Dir::parse(e.dir.as_str()).unwrap();
                let to_id = obj_by_id.get(&e.to).unwrap();

                (dir, *to_id)
            }).collect();

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

