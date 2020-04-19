use super::container::Container;
use super::item::*;
use super::mob::*;
use crate::errors::{AsResult, Error, Result};
use crate::game::Outputs;
use crate::game::{comm, inventory};
use commons::PlayerId;

#[derive(Debug, Clone, PartialEq)]
pub enum PickUpError {
    Stuck,
    NotInventory,
    Other,
}

pub fn do_pickup(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    item_id: ItemId,
    inventory_id: Option<ItemId>,
) -> std::result::Result<(), PickUpError> {
    let item = container.items.get(item_id).ok_or(PickUpError::Other)?;
    let room_id = container.locations.get(mob_id).ok_or(PickUpError::Other)?;
    let mob_label = container.labels.get_label_f(mob_id);
    let item_label = container.labels.get_label_f(item_id);

    if item.flags.is_stuck {
        outputs.private(mob_id, comm::pick_fail_item_is_stuck(item_label));
        return Err(PickUpError::Stuck);
    }

    match inventory_id {
        Some(inventory_id) => {
            let inventory_label = container.labels.get_label_f(inventory_id);
            let inventory_item = container
                .items
                .get(inventory_id)
                .ok_or(PickUpError::Other)?;

            if !inventory_item.flags.is_inventory {
                outputs.private(
                    mob_id,
                    comm::pick_fail_storage_is_not_inventory(inventory_label),
                );
                return Err(PickUpError::NotInventory);
            }

            outputs.private(mob_id, comm::pick_player_from(inventory_label, item_label));
            outputs.broadcast(
                Some(mob_id),
                room_id,
                comm::pick_from(mob_label, inventory_label, item_label),
            );
        }
        None => {
            outputs.private(mob_id, comm::pick_player_from_room(item_label));
            outputs.broadcast(
                Some(mob_id),
                room_id,
                comm::pick_from_room(mob_label, item_label),
            );
        }
    }

    inventory::add(container, item_id, mob_id).map_err(|_error| {
        let item_label = container.labels.get_label_f(item_id);

        outputs.private(mob_id, comm::pick_fail_storage_is_not_inventory(item_label));

        PickUpError::Other
    })
}

/// As a humanoid entity in mud, try to equip a item
pub fn do_equip(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    item_id: ItemId,
) -> Result<()> {
    let _item = container.items.get(item_id).as_result()?;
    let item_label = container.labels.get_label_f(item_id);

    // check if mob own the item
    let has_item = container
        .locations
        .list_at(mob_id)
        .find(|id| *id == item_id)
        .is_some();

    let item = container.items.get(item_id);

    if !has_item || item.is_none() {
        outputs.private(mob_id, comm::equip_what());
        return Err(Error::NotFoundFailure);
    }

    // check if can be equipped
    let item = item.unwrap();
    let can_be_equipped = item.weapon.is_some() || item.armor.is_some();

    if !can_be_equipped {
        outputs.private(mob_id, comm::equip_item_invalid(item_label));
        return Err(Error::InvalidArgumentFailure);
    }

    // TODO: remove old equip in same place?
    container.equips.add(mob_id, item_id);

    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let room_id = container.locations.get(mob_id).as_result()?;
    outputs.private(mob_id, comm::equip_player_from_room(item_label));
    outputs.broadcast(
        Some(mob_id),
        room_id,
        comm::equip_from_room(mob_label, item_label),
    );
    Ok(())
}

pub fn do_strip(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    item_id: ItemId,
) -> Result<()> {
    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let room_id = container.locations.get(mob_id).as_result()?;
    let item_label = container.labels.get_label(item_id).as_result()?;

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);

    outputs.private(mob_id, comm::strip_player_from_room(item_label));
    outputs.broadcast(
        Some(mob_id),
        room_id,
        comm::strip_from_room(mob_label, item_label),
    );
    Ok(())
}

pub fn do_drop(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    item_id: ItemId,
) -> Result<()> {
    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let room_id = container.locations.get(mob_id).as_result()?;
    let item_label = container.labels.get_label(item_id).as_result()?;

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);
    container.locations.set(item_id, room_id);

    outputs.private(mob_id, comm::drop_item(item_label));
    outputs.broadcast(
        Some(mob_id),
        room_id,
        comm::drop_item_others(mob_label, item_label),
    );
    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::controller::OutputsBuffer;
    use crate::game::builder;
    use crate::game::container::Container;
    use crate::game::item::ItemId;
    use crate::game::mob::MobId;
    use crate::game::room::RoomId;

    pub struct TestScenery {
        pub container: Container,
        pub room_id: RoomId,
        pub container_id: ItemId,
        pub item1_id: ItemId,
        pub item2_id: ItemId,
        pub mob_id: MobId,
    }

    pub fn setup() -> TestScenery {
        let mut container = Container::new();
        let room_id = builder::add_room(&mut container, "test_room");

        // TODO: remove hack when we use proper item builder
        let container_id = builder::add_item(&mut container, "container1", room_id);
        {
            let mut item = container.items.remove(container_id).unwrap();
            item.flags.is_stuck = true;
            item.flags.is_inventory = true;
            container.items.add(item);
        }

        let item1_id = builder::add_item(&mut container, "item1", room_id);
        let item2_id = builder::add_item(&mut container, "item2", container_id);

        let mob_id = builder::add_mob(&mut container, "mob", room_id);

        TestScenery {
            container,
            room_id,
            container_id,
            item1_id,
            item2_id,
            mob_id,
        }
    }

    #[test]
    pub fn test_do_pickup_should_fail_if_inventory_is_not_inventory() {
        let mut outputs = OutputsBuffer::new();
        let mut scenery = setup();
        let result = do_pickup(
            &mut scenery.container,
            &mut outputs,
            scenery.mob_id,
            scenery.container_id,
            None,
        );
        assert_eq!(result.err().unwrap(), PickUpError::Stuck);
    }

    #[test]
    pub fn test_do_pickup_should_fail_if_item_is_stuck() {
        let mut outputs = OutputsBuffer::new();
        let mut scenery = setup();
        let result = do_pickup(
            &mut scenery.container,
            &mut outputs,
            scenery.mob_id,
            scenery.item1_id,
            Some(scenery.item2_id),
        );
        assert_eq!(result.err().unwrap(), PickUpError::NotInventory);
    }
}
