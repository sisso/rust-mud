use super::container::Container;
use super::item::*;
use super::mob::*;
use crate::errors::{AsResult, Error, Result};
use crate::game::outputs::Outputs;
use crate::game::{comm, inventory_service};
use commons::PlayerId;

#[derive(Debug)]
pub enum PickUpError {
    Stuck,
    NotInventory,
    Full,
    Other(Error),
}

impl From<Error> for PickUpError {
    fn from(e: Error) -> Self {
        PickUpError::Other(e)
    }
}

const NOT_FOUND_EXCEPTION: PickUpError = PickUpError::Other(Error::NotFoundException);

// TODO: why this method receive a inventory_id? Looks like it is not needed, we can use
//       item_location_id and decide
pub fn do_pickup(
    container: &mut Container,
    mob_id: MobId,
    item_id: ItemId,
    inventory_id: Option<ItemId>,
) -> std::result::Result<(), PickUpError> {
    let item = container.items.get(item_id).ok_or(NOT_FOUND_EXCEPTION)?;
    let item_location_id = container.locations.get(item_id);
    let room_id = container.locations.get(mob_id).ok_or(NOT_FOUND_EXCEPTION)?;
    let mob_label = container.labels.get_label_f(mob_id);
    let item_label = container.labels.get_label_f(item_id);

    if item.flags.is_stuck {
        container
            .outputs
            .private(mob_id, comm::pick_fail_item_is_stuck(item_label));
        return Err(PickUpError::Stuck);
    }

    if let Some(weight) = item.weight {
        let can_add_weight = !inventory_service::can_add_weight(container, mob_id, weight)
            .map_err(|e| PickUpError::Other(e))?;

        if can_add_weight {
            container
                .outputs
                .private(mob_id, comm::pick_fail_storage_is_not_inventory(item_label));
            return Err(PickUpError::Full);
        }
    }

    match inventory_id {
        Some(inventory_id) => {
            let inventory_label = container.labels.get_label_f(inventory_id);
            let inventory_item = container
                .items
                .get(inventory_id)
                .ok_or(NOT_FOUND_EXCEPTION)?;

            if !inventory_item.flags.is_inventory {
                container.outputs.private(
                    mob_id,
                    comm::pick_fail_storage_is_not_inventory(inventory_label),
                );
                return Err(PickUpError::NotInventory);
            }

            container
                .outputs
                .private(mob_id, comm::pick_player_from(inventory_label, item_label));
            container.outputs.broadcast(
                Some(mob_id),
                room_id,
                comm::pick_from(mob_label, inventory_label, item_label),
            );
        }
        None => {
            container
                .outputs
                .private(mob_id, comm::pick_player_from_room(item_label));
            container.outputs.broadcast(
                Some(mob_id),
                room_id,
                comm::pick_from_room(mob_label, item_label),
            );
        }
    }

    inventory_service::add(container, item_id, mob_id).map_err(|error| {
        let item_label = container.labels.get_label_f(item_id);

        container
            .outputs
            .private(mob_id, comm::pick_fail_storage_is_not_inventory(item_label));

        PickUpError::Other(error)
    })?;

    // update inventory of new and old location of the item
    inventory_service::update_inventory_weight(container, mob_id)?;
    if let Some(item_location_id) = item_location_id {
        inventory_service::update_inventory_weight(container, item_location_id)?;
    }

    Ok(())
}

/// As a humanoid entity in mud, try to equip a item
pub fn do_equip(container: &mut Container, mob_id: MobId, item_id: ItemId) -> Result<()> {
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
        container.outputs.private(mob_id, comm::equip_what());
        return Err(Error::NotFoundFailure);
    }

    // check if can be equipped
    let item = item.unwrap();
    let can_be_equipped = item.weapon.is_some() || item.armor.is_some();

    if !can_be_equipped {
        container
            .outputs
            .private(mob_id, comm::equip_item_invalid(item_label));
        return Err(Error::InvalidArgumentFailure);
    }

    // TODO: remove old equip in same place?
    container.equips.add(mob_id, item_id);

    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let room_id = container.locations.get(mob_id).as_result()?;
    container
        .outputs
        .private(mob_id, comm::equip_player_from_room(item_label));
    container.outputs.broadcast(
        Some(mob_id),
        room_id,
        comm::equip_from_room(mob_label, item_label),
    );
    Ok(())
}

pub fn do_strip(container: &mut Container, mob_id: MobId, item_id: ItemId) -> Result<()> {
    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let room_id = container.locations.get(mob_id).as_result()?;
    let item_label = container.labels.get_label(item_id).as_result()?;

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);

    container
        .outputs
        .private(mob_id, comm::strip_player_from_room(item_label));
    container.outputs.broadcast(
        Some(mob_id),
        room_id,
        comm::strip_from_room(mob_label, item_label),
    );
    Ok(())
}

pub fn do_drop(container: &mut Container, mob_id: MobId, item_id: ItemId) -> Result<()> {
    let mob_label = container.labels.get_label(mob_id).as_result()?;
    let room_id = container.locations.get(mob_id).as_result()?;
    let item_label = container.labels.get_label(item_id).as_result()?;
    let item_location_id = container.locations.get(item_id);

    // strip if is in use
    let _ = container.equips.strip(mob_id, item_id);
    container.locations.set(item_id, room_id);

    container
        .outputs
        .private(mob_id, comm::drop_item(item_label));

    container.outputs.broadcast(
        Some(mob_id),
        room_id,
        comm::drop_item_others(mob_label, item_label),
    );

    // update inventory weights
    inventory_service::update_inventory_weight(container, mob_id)?;

    if let Some(item_location_id) = item_location_id {
        inventory_service::update_inventory_weight(container, item_location_id)?;
    }

    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::game::builder;
    use crate::game::container::Container;
    use crate::game::item::ItemId;
    use crate::game::loader::dto::{ItemData, ItemFlagsData, ObjData, StaticId};
    use crate::game::mob::MobId;
    use crate::game::obj::Obj;
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
        let mut scenery = setup();
        let result = do_pickup(
            &mut scenery.container,
            scenery.mob_id,
            scenery.container_id,
            None,
        );

        match result {
            Err(PickUpError::Stuck) => {}
            other => panic!("not expected {:?}", other),
        }
    }

    #[test]
    pub fn test_do_pickup_should_fail_if_item_is_stuck() {
        let mut scenery = setup();
        let result = do_pickup(
            &mut scenery.container,
            scenery.mob_id,
            scenery.item1_id,
            Some(scenery.item2_id),
        );

        match result {
            Err(PickUpError::NotInventory) => {}
            other => panic!("not expected {:?}", other),
        }
    }

    #[test]
    fn test_do_pickup_should_limit_until_location_max_weight() -> Result<()> {
        let mut scenery = setup();

        let container = &mut scenery.container;

        builder::set_mob_max_carry_weight(container, scenery.mob_id, 10.0);

        for _ in 0..20 {
            let item_id = builder::add_item(container, "item", scenery.room_id);
            builder::set_item_weight(container, item_id, 1.0);

            let result = do_pickup(container, scenery.mob_id, item_id, None);

            if result.is_err() {
                break;
            }
        }

        let inventory_list = inventory_service::get_inventory_list(
            &container.locations,
            &container.items,
            scenery.mob_id,
        );
        assert_eq!(inventory_list.count(), 10);

        let inventory = container.inventories.get(scenery.mob_id).unwrap();
        assert_eq!(inventory.max_weight, 10.0);
        assert_eq!(inventory.current_weight, 10.0);

        Ok(())
    }

    #[test]
    fn test_drop_should_update_weight_if_location_has_inventory() -> Result<()> {
        let mut scenery = setup();

        builder::set_mob_max_carry_weight(&mut scenery.container, scenery.mob_id, 10.0);

        let item_id = builder::add_item(&mut scenery.container, "item", scenery.room_id);
        builder::set_item_weight(&mut scenery.container, item_id, 5.0);
        do_pickup(&mut scenery.container, scenery.mob_id, item_id, None).unwrap();

        let item_id = builder::add_item(&mut scenery.container, "item", scenery.room_id);
        builder::set_item_weight(&mut scenery.container, item_id, 5.0);
        do_pickup(&mut scenery.container, scenery.mob_id, item_id, None).unwrap();

        super::do_drop(&mut scenery.container, scenery.mob_id, item_id)?;

        let inventory_list = inventory_service::get_inventory_list(
            &scenery.container.locations,
            &scenery.container.items,
            scenery.mob_id,
        );
        assert_eq!(inventory_list.count(), 1);

        let inventory = scenery.container.inventories.get(scenery.mob_id).unwrap();
        assert_eq!(inventory.max_weight, 10.0);
        assert_eq!(inventory.current_weight, 5.0);

        Ok(())
    }
}
