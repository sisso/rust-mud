use crate::errors::*;
use crate::game::container::Container;
use crate::game::item::{Item, ItemId, ItemRepository, Weight};
use crate::game::labels::Labels;
use crate::game::loader::dto::StaticId;
use crate::game::loader::Loader;
use crate::game::location;
use crate::game::location::{LocationId, Locations};
use crate::game::prices::Money;
use commons::ObjId;
use logs::*;

// TODO: Merge common code related with money
// TODO: Money change should just be a +/- function
// TODO: merge items by count should not be exclusive to money

pub fn update_all_current_inventory(container: &mut Container) {
    let ids: Vec<ObjId> = container.inventories.list_ids().cloned().collect();
    for id in ids {
        if let Err(e) = update_inventory_weight(container, id) {
            warn!("{:?} error when computing weight {:?}", id, e);
        }
    }
}

pub fn compute_total_weight(items: &Vec<&Item>) -> Weight {
    items.iter().map(|item| item.total_weight()).sum()
}

pub fn can_add_weight_by_prefab(container: &Container, obj_id: ObjId, static_id: StaticId) -> bool {
    let weight = match container.loader.get_prefab_weight(static_id) {
        Some(w) => w,
        None => return true,
    };

    let location_inventory = container.inventories.get(obj_id);
    location_inventory
        .map(|inv| inv.can_add(weight))
        .unwrap_or(false)
}

pub fn can_add_weight(
    container: &Container,
    location_id: LocationId,
    weight: Weight,
) -> Result<bool> {
    let location_inventory = container.inventories.get(location_id);
    Ok(location_inventory
        .map(|inv| inv.can_add(weight))
        .unwrap_or(false))
}

pub fn update_inventory_weight(container: &mut Container, location_id: LocationId) -> Result<()> {
    let locations = &container.locations;
    let items = &container.items;
    let inventories = &mut container.inventories;

    let inventory = if let Some(inv) = inventories.get_mut(location_id) {
        inv
    } else {
        return Ok(());
    };

    let total_weight = locations
        .list_deep_at(location_id)
        .into_iter()
        .flat_map(|id| items.get(id).and_then(|item| item.weight))
        .sum();

    inventory.current_weight = Some(total_weight);

    Ok(())
}

pub fn add(container: &mut Container, item_id: ItemId, location_id: LocationId) -> Result<()> {
    let item = container.items.get(item_id).as_result()?;

    // Money items can cause to be merge if target inventory already contain it, this means that
    // previous item can get deleted
    if item.flags.is_money {
        let amount = container
            .items
            .get(item_id)
            .ok_or(Error::NotFoundFailure)?
            .amount
            .into();

        add_money_with_item(container, location_id, amount, Some(item_id))
    } else {
        container.locations.set(item_id, location_id);
        Ok(())
    }
}

pub fn add_money(container: &mut Container, obj_id: ObjId, amount: Money) -> Result<()> {
    // check if already have any money item and append, otherwise find one in loader and spawn
    add_money_with_item(container, obj_id, amount, None)
}

pub fn get_money(container: &Container, obj_id: ObjId) -> Result<Money> {
    Ok(get_money_id(container, obj_id)
        .and_then(|id| container.items.get(id))
        .map(|item| Money(item.amount))
        .unwrap_or(Money(0)))
}

pub fn get_money_id(container: &Container, obj_id: ObjId) -> Option<ItemId> {
    container
        .locations
        .list_at(obj_id)
        .flat_map(|id| container.items.get(id))
        .filter(|item| item.flags.is_money)
        .map(|item| item.id)
        .next()
}

/// return the new money amount
pub fn remove_money(container: &mut Container, obj_id: ObjId, amount: Money) -> Result<Money> {
    let item_id = get_money_id(container, obj_id).as_result()?;

    let item = container
        .items
        .get_mut(item_id)
        .ok_or(Error::InvalidStateException)?;

    if item.amount < amount.as_u32() {
        return Err(Error::InvalidArgumentFailure);
    }

    if item.amount == amount.as_u32() {
        container.remove(item_id);
        Ok(Money(0))
    } else {
        item.amount -= amount.as_u32();
        Ok(item.amount.into())
    }
}

fn add_money_with_item(
    container: &mut Container,
    inventory_id: ObjId,
    amount: Money,
    provided_item_id: Option<ItemId>,
) -> Result<()> {
    let inventory_item_id =
        get_inventory_list(&container.locations, &container.items, inventory_id)
            .into_iter()
            .filter(|item| item.flags.is_money)
            .map(|item| item.id)
            .next();

    match (inventory_item_id, provided_item_id) {
        (Some(item_id), Some(provided_item_id)) => {
            let item = container
                .items
                .get_mut(item_id)
                .expect("Money was created but is not a item");
            item.amount += amount.as_u32();

            // since we add to a already existent item, we don't need this anymore
            container.remove(provided_item_id);

            debug!(
                "{:?} receive {:?} money by merging inventory money",
                inventory_id, amount
            );

            Ok(())
        }
        (Some(item_id), None) => {
            let item = container
                .items
                .get_mut(item_id)
                .expect("Money was created but is not a item");
            item.amount += amount.as_u32();
            debug!(
                "{:?} receive {:?} money by updating inventory money",
                inventory_id, amount
            );
            Ok(())
        }
        (None, Some(item_id)) => {
            container.locations.set(item_id, inventory_id);
            container.ownership.remove_owner(item_id);
            debug!(
                "{:?} receive {:?} money by adding money to inventory",
                inventory_id, amount
            );
            Ok(())
        }
        (None, None) => {
            let static_id = container.config.money_id.expect("money_id not defined");
            let item_id = Loader::spawn_at(container, static_id, inventory_id)?;
            let item = container
                .items
                .get_mut(item_id)
                .expect("Money was created but is not a item");
            item.amount = amount.as_u32();
            debug!(
                "{:?} receive {:?} money by creating money in inventory",
                inventory_id, amount
            );
            Ok(())
        }
    }
}

pub fn move_all(locations: &mut Locations, from: ObjId, to: ObjId) {
    let list: Vec<_> = locations.list_at(from).collect();
    for i in list {
        locations.set(i, to);
    }
}

pub fn get_inventory_list<'a>(
    locations: &'a Locations,
    items: &'a ItemRepository,
    obj_id: ObjId,
) -> impl Iterator<Item = &'a Item> + 'a {
    locations.list_at(obj_id).flat_map(move |id| items.get(id))
}

pub fn search(
    labels: &Labels,
    locations: &Locations,
    items: &ItemRepository,
    location_id: ObjId,
    label: &str,
) -> Vec<ItemId> {
    location::search_at(labels, locations, location_id, label)
        .into_iter()
        .filter(|obj_id| items.exists(*obj_id))
        .collect()
}

pub fn search_one(
    labels: &Labels,
    locations: &Locations,
    items: &ItemRepository,
    location_id: ObjId,
    label: &str,
) -> Option<ItemId> {
    search(labels, locations, items, location_id, label)
        .into_iter()
        .next()
}
