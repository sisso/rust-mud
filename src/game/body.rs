use super::comm;
use super::item::*;
use super::mob::*;
use super::container::*;
use super::controller::Outputs;

pub fn create_body(container: &mut Container, outputs: &mut Outputs, mob_id: &MobId) {
    let item_id = container.items.next_item_id();
    let mob = container.mobs.get(mob_id);
    let room_id = mob.room_id;

    let item = Item {
        id: item_id,
        typ: ITEM_TYPE_BODY,
        label: format!("{} body", mob.label).to_string(),
    };

    let msg = comm::item_body_appears_in_room(&item);

    container.items.add_to_room(item, room_id);

    outputs.room_all(room_id, msg);
}
