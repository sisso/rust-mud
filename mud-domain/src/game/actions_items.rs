use super::container::Container;
use super::item::*;
use super::mob::*;
use crate::game::comm;
use crate::game::Outputs;
use commons::{PlayerId, AsResult, UResult};

#[derive(Debug, Clone,PartialEq)]
pub enum PickUpError {
    Stuck,
    NotInventory,
    Other,
}

pub fn do_pickup(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId, inventory_id: Option<ItemId>) -> Result<(),PickUpError> {
    let _mob = container.mobs.get(mob_id).ok_or(PickUpError::Other)?;
    let item = container.items.get(item_id).ok_or(PickUpError::Other)?;
    let room_id = container.locations.get(mob_id).ok_or(PickUpError::Other)?;
    let mob_label = container.labels.get_label_f(mob_id);
    let item_label = container.labels.get_label_f(item_id);

    if item.is_stuck {
        outputs.private_opt(player_id, comm::pick_fail_item_is_stuck(item_label));
        return Err(PickUpError::Stuck);
    }

    match inventory_id {
        Some(inventory_id) => {
            let inventory_label = container.labels.get_label_f(inventory_id);
            let inventory = container.items.get(inventory_id).ok_or(PickUpError::Other)?;

            if !inventory.is_inventory {
                outputs.private_opt(player_id, comm::pick_fail_storage_is_not_inventory(inventory_label));
                return Err(PickUpError::NotInventory);
            }

            outputs.private_opt(player_id, comm::pick_player_from(inventory_label, item_label));
            outputs.room_opt(player_id, room_id, comm::pick_from(mob_label, inventory_label, item_label));
        }
        None => {
            outputs.private_opt(player_id, comm::pick_player_from_room(item_label));
            outputs.room_opt(player_id, room_id, comm::pick_from_room(mob_label, item_label));
        }
    }

    container.locations.set(item_id, mob_id);

    Ok(())
}

/// As a humanoid entity in mud, try to equip a item
pub fn do_equip(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(), ()> {
    let _item = container.items.get(item_id).as_result()?;
    let item_label = container.labels.get_label_f(item_id);

    // check if mob own the item
    let has_item = container.locations.list_at(mob_id)
        .find(|id| *id == item_id)
        .is_some();

    let item = container.items.get(item_id);

    if !has_item || item.is_none() {
        outputs.private_opt(player_id, comm::equip_what());
        return Err(());
    }

    // check if can be equipped
    let item = item.unwrap();
    let can_be_equipped = item.weapon.is_some() || item.armor.is_some();

    if !can_be_equipped {
        outputs.private_opt(player_id, comm::equip_item_invalid(item_label));
        return Err(());
    }

    // TODO: remove old equip in sample place?
    container.equips.add(mob_id, item_id);

    let mob_label = container.labels.get_label_f(mob_id);
    let room_id = container.locations.get(mob_id).as_result()?;
    outputs.private_opt(player_id, comm::equip_player_from_room(item_label));
    outputs.room_opt(player_id, room_id,comm::equip_from_room(mob_label, item_label));
    Ok(())
}

pub fn do_strip(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> Result<(),()> {
    let mob_label = container.labels.get_label_f(mob_id);
    let room_id = container.locations.get(mob_id).as_result()?;
    let item_label = container.labels.get_label_f(item_id);

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);

    outputs.private_opt(player_id, comm::strip_player_from_room(item_label));
    outputs.room_opt(player_id, room_id, comm::strip_from_room(mob_label, item_label));
    Ok(())
}

pub fn do_drop(container: &mut Container, outputs: &mut dyn Outputs, player_id: Option<PlayerId>, mob_id: MobId, item_id: ItemId) -> UResult {
    let _mob = container.mobs.get(mob_id).as_result()?;
    let mob_label = container.labels.get_label_f(mob_id);
    let room_id = container.locations.get(mob_id).as_result()?;
    let _item = container.items.get(item_id).as_result()?;
    let item_label = container.labels.get_label_f(item_id);

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);
    container.locations.set(item_id, room_id);

    outputs.private_opt(player_id, comm::drop_item(item_label));
    outputs.room_opt(player_id, room_id, comm::drop_item_others(mob_label, item_label));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::test::setup;
    use crate::game::OutputsBuffer;

    #[test]
    pub fn test_do_pickup_should_fail_if_inventory_is_not_inventory() {
        let mut outputs = OutputsBuffer::new();
        let mut scenery = setup();
        let result = do_pickup(&mut scenery.container, &mut outputs, None, scenery.mob_id, scenery.container_id, None);
        assert_eq!(result.err().unwrap(),  PickUpError::Stuck);
    }

    #[test]
    pub fn test_do_pickup_should_fail_if_item_is_stuck() {
        let mut outputs = OutputsBuffer::new();
        let mut scenery = setup();
        let result = do_pickup(&mut scenery.container, &mut outputs, None, scenery.mob_id, scenery.item1_id, Some(scenery.item2_id));
        assert_eq!(result.err().unwrap(), PickUpError::NotInventory);
    }
}
