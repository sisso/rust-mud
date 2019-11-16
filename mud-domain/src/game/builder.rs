use crate::game::room::{RoomId, Room};
use crate::game::container::Container;
use crate::game::item::{Item, ITEM_KIND_UNDEFINED, ItemId, ItemPrefabId};
use commons::ObjId;
use crate::game::mob::{Mob, MobId, MobPrefabId};
use crate::game::labels::Label;

/*
Builder methods to instantiate game components from code. Main use for testing.
*/

pub fn add_room(container: &mut Container, label: &str, desc: &str) -> RoomId {
    let room_id = container.objects.create();

    container.rooms.add(Room::new(room_id));

    container.labels.set(Label {
        id: room_id,
        label: label.to_string(),
        code: label.to_string(),
        desc: desc.to_string()
    });

    room_id
}

pub fn add_item(container: &mut Container, label: &str, location_id: ObjId) -> ItemId {
    let item_id = container.objects.create();
    container.items.add(Item {
        id: item_id,
        kind: ITEM_KIND_UNDEFINED,
        decay: None,
        amount: 1,
        item_def_id: None,
        weapon: None,
        armor: None,
        is_inventory: false,
        is_stuck: false
    });

    container.labels.set(Label {
        id: item_id,
        label: label.to_string(),
        code: label.to_string(),
        desc: "".to_string(),
    });

    container.locations.set(item_id, location_id);

    item_id
}

pub fn add_mob(container: &mut Container, label: &str, location_id: RoomId) -> MobId {
    let id = container.objects.create();
    container.mobs.add(Mob::new(id));

    container.labels.set(Label {
        id,
        label: label.to_string(),
        code: label.to_string(),
        desc: "".to_string(),
    });

    container.locations.set(id, location_id);

    id
}

pub fn add_item_from_prefab(container: &mut Container, item_prefab_id: ItemPrefabId, location_id: ObjId) -> ItemId {
    let item_id = container.objects.create();
    let prefab = container.items.get_prefab(&item_prefab_id);

    let mut item = Item::new(
        item_id,
        prefab.kind,
    );

    item.amount = prefab.amount;
    item.item_def_id = Some(item_prefab_id);
    item.weapon = prefab.weapon.clone();
    item.armor = prefab.armor.clone();
    item.is_inventory = prefab.is_inventory;
    item.is_stuck = prefab.is_stuck;

    container.labels.set(Label {
        id: item_id,
        label: prefab.label.clone(),
        code: prefab.label.clone(),
        desc: prefab.label.clone(),
    });
    container.items.add(item);
    container.locations.set(item_id, location_id);

    item_id
}

pub fn add_mob_from_prefab(container: &mut Container, mob_prefab_id: MobPrefabId, room_id: RoomId) -> Result<MobId,()> {
    let prefab = container.mobs.get_mob_prefab(mob_prefab_id).clone();

    // create mob
    let mob_id = container.objects.create();

    // add items
    for item_prefab_id in prefab.inventory {
        let _= add_item_from_prefab(container, item_prefab_id, mob_id);
    }

    // instantiate
    let mut mob = Mob::new(mob_id);
    mob.attributes = prefab.attributes;
    container.mobs.add(mob);

    container.locations.set(mob_id, room_id);
    container.labels.set(Label::new(mob_id, prefab.label.as_str()));

    Ok(mob_id)
}


