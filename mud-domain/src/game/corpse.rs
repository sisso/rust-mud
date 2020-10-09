use super::comm;
use super::container::*;
use super::item::*;
use super::mob::*;
use crate::game::inventory_service;
use crate::game::labels::Label;
use crate::game::triggers::*;
use commons::DeltaTime;
use logs::*;

const DECAY_TIME: DeltaTime = DeltaTime(20.0);

pub fn create_corpse(container: &mut Container, mob_id: MobId) {
    let corpse_id = container.objects.create();
    let room_id = container.locations.get(mob_id).unwrap();
    let mob_label = container.labels.get_label(mob_id).unwrap();

    let mut corpse = Item::new(corpse_id);
    corpse.flags.is_corpse = true;
    corpse.flags.is_inventory = true;
    container.items.add(corpse);

    container.locations.set(corpse_id, room_id);

    let corpse_label = format!("{} corpse", mob_label);
    let corpse_desc = format!("The corpse of {} laying in the ground", mob_label);

    container.labels.add(Label {
        id: corpse_id,
        label: corpse_label.clone(),
        code: "corpse".to_string(),
        desc: corpse_desc.to_string(),
    });
    inventory_service::move_all(&mut container.locations, mob_id, corpse_id);

    container.timer.schedule(
        container.time.total + DECAY_TIME,
        Event::Obj {
            kind: EventKind::Decay,
            obj_id: corpse_id,
        },
    );

    debug!(
        "{:?} corpse of {:?} created at {:?}",
        corpse_id, mob_id, room_id
    );

    let msg = comm::item_corpse_appears_in_room(corpse_label.as_str());
    container.outputs.broadcast(None, room_id, msg);
}
