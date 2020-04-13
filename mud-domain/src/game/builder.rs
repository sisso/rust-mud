use crate::game::container::Container;
use crate::game::item::{Item, ItemFlags, ItemId};
use crate::game::labels::Label;
use crate::game::mob::{Mob, MobId};
use crate::game::room::{Room, RoomId};
use commons::ObjId;

/*
Builder methods to instantiate game components from code. Main use for testing.
*/

pub fn add_room(container: &mut Container, label: &str) -> RoomId {
    let room_id = container.objects.create();

    container.rooms.add(Room::new(room_id));

    container.labels.add(Label {
        id: room_id,
        label: label.to_string(),
        // TODO: use autocode
        code: label.to_string(),
        desc: label.to_string(),
    });

    room_id
}

pub fn add_item(container: &mut Container, label: &str, location_id: ObjId) -> ItemId {
    let item_id = container.objects.create();

    let item = Item::new(item_id);
    container.items.add(item);

    container.labels.add(Label::new(item_id, label));
    container.locations.set(item_id, location_id);

    item_id
}

pub fn add_mob(container: &mut Container, label: &str, location_id: RoomId) -> MobId {
    let id = container.objects.create();
    container.mobs.add(Mob::new(id));
    container.labels.add(Label::new(id, label));
    container.locations.set(id, location_id);
    id
}
