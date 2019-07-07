use super::item::*;
use super::inventory;
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

    container.items.add(item);
//    container.add_item_to_room(container, item_id, room_id);
}
