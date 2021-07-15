use crate::game::container::Container;
use crate::game::domain::Dir;
use crate::game::inventory::Inventory;
use crate::game::item::{Item, ItemFlags, ItemId};
use crate::game::labels::Label;
use crate::game::loader::dto::{ItemData, ItemFlagsData, ObjData, StaticId, TagsData};
use crate::game::location::LocationId;
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

pub fn add_container(
    container: &mut Container,
    label: &str,
    location_id: ObjId,
    stuck: bool,
) -> ItemId {
    let item_id = container.objects.create();

    let mut item = Item::new(item_id);
    item.flags.is_stuck = stuck;
    item.flags.is_inventory = true;
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

pub fn set_item_weight(container: &mut Container, item_id: ItemId, weight: f32) {
    container.items.get_mut(item_id).unwrap().weight = Some(weight);
}

pub fn set_mob_max_carry_weight(container: &mut Container, obj_id: ObjId, max_weight: f32) {
    let mut inventory = Inventory::new(obj_id);
    inventory.max_weight = Some(max_weight);
    container.inventories.add(inventory).unwrap();
}

pub fn add_portal(container: &mut Container, from: RoomId, to: RoomId, dir: Dir) {
    container
        .rooms
        .update(from, |r| {
            r.exits.push((dir, to));
        })
        .unwrap();
    container
        .rooms
        .update(to, |r| {
            r.exits.push((dir.inv(), from));
        })
        .unwrap();
}

pub fn add_money(container: &mut Container) -> StaticId {
    match container.config.money_id {
        Some(id) => id,
        None => {
            let mut data = ObjData::new();
            data.label = Some("money".to_string());

            let mut item_flag_data = ItemFlagsData::new();
            item_flag_data.money = Some(true);

            let mut item_data = ItemData::new();
            item_data.amount = Some(1);
            item_data.flags = Some(item_flag_data);

            data.item = Some(item_data);
            data.tags = Some(TagsData {
                values: vec!["money".to_string()],
            });

            let id = container
                .loader
                .add_prefab(data)
                .expect("fail to add money prefab");

            container.config.money_id = Some(id);

            id
        }
    }
}
