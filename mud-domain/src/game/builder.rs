use crate::game::room::{RoomId, Room};
use crate::game::container::Container;
use crate::game::item::{Item, ITEM_KIND_UNDEFINED, ItemId};
use commons::ObjId;

pub fn add_room(container: &mut Container, label: &str, desc: &str) -> RoomId {
    let room_id = container.objects.insert();
    container.rooms.add(Room {
        id: room_id,
        label: label.to_string(),
        desc: desc.to_string(),
        exits: vec![]
    });
    room_id
}

pub fn add_item(container: &mut Container, label: &str, location_id: ObjId) -> ItemId {
    let item_id = container.objects.insert();
    container.items.add(Item {
        id: item_id,
        kind: ITEM_KIND_UNDEFINED,
        label: label.to_string(),
        decay: None,
        amount: 1,
        item_def_id: None,
        weapon: None,
        armor: None
    });

    container.locations.set(item_id, location_id);

    item_id
}
