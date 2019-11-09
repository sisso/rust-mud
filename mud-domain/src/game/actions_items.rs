use super::container::Container;
use super::item::*;
use super::mob::*;
use crate::game::comm;
use crate::game::Outputs;
use commons::PlayerId;

pub fn do_pickup(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId, inventory_id: Option<ItemId>) -> Result<(),()> {
    let mob = container.mobs.get(mob_id)?;
    let item = container.items.get(item_id)?;
    let room_id = container.locations.get(mob_id)?;

    match inventory_id {
        Some(inventory_id) => {
            let inventory = container.items.get(inventory_id)?;
            outputs.private_opt(player_id, comm::pick_player_from(inventory.label.as_str(), item.label.as_str()));
            outputs.room_opt(player_id, room_id, comm::pick_from(mob.label.as_str(), inventory.label.as_str(), item.label.as_str()));
        }
        None => {
            outputs.private_opt(player_id, comm::pick_player_from_room(item.label.as_str()));
            outputs.room_opt(player_id, room_id, comm::pick_from_room(mob.label.as_str(), item.label.as_str()));
        }
    }

    container.locations.set(item_id, mob_id);

    Ok(())
}

/// As a humanoid entity in mud, try to equip a item
pub fn do_equip(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(), ()> {
    let item = container.items.get(item_id);

    // check if mob own the item
    let has_item = container.locations.list_at(mob_id)
        .find(|id| *id == item_id)
        .is_some();

    let item = container.items.get(item_id).ok();

    if !has_item || item.is_none() {
        outputs.private_opt(player_id, comm::equip_what());
        return Err(());
    }

    // check if can be equipped
    let item = item.unwrap();
    let can_be_equipped = item.weapon.is_some() || item.armor.is_some();

    if !can_be_equipped {
        outputs.private_opt(player_id, comm::equip_item_invalid(item.label.as_str()));
        return Err(());
    }

    // TODO: remove old equip in sample place?
    container.equips.add(mob_id, item_id);

    let mob = container.mobs.get(mob_id)?;
    let room_id = container.locations.get(mob_id)?;
    outputs.private_opt(player_id, comm::equip_player_from_room(item.label.as_str()));
    outputs.room_opt(player_id, room_id,comm::equip_from_room(mob.label.as_str(), item.label.as_str()));
    Ok(())
}

pub fn do_strip(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(),()> {
    let mob = container.mobs.get(mob_id)?;
    let room_id = container.locations.get(mob_id)?;
    let item = container.items.get(item_id)?;

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);

    outputs.private_opt(player_id, comm::strip_player_from_room(item.label.as_str()));
    outputs.room_opt(player_id, room_id, comm::strip_from_room(mob.label.as_str(), item.label.as_str()));
    Ok(())
}

pub fn do_drop(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(),()> {
    let mob = container.mobs.get(mob_id)?;
    let room_id = container.locations.get(mob_id)?;
    let item = container.items.get(item_id)?;

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);
    container.locations.set(item_id, room_id);

    outputs.private_opt(player_id, comm::drop_item(item.label.as_str()));
    outputs.room_opt(player_id, room_id, comm::drop_item_others(mob.label.as_str(), item.label.as_str()));
    Ok(())
}


#[test]
pub fn test1() {
}
