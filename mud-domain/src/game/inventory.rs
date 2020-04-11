use crate::errors::*;
use crate::game::container::Container;
use crate::game::item::{Item, ItemId, ItemRepository};
use crate::game::labels::Labels;
use crate::game::loader::Loader;
use crate::game::location;
use crate::game::location::{LocationId, Locations};
use crate::game::prices::Money;
use commons::ObjId;
use logs::*;

// TODO: Merge common code related with money

/// Money items can cause to be merge if target inventory already contain it, this means that
/// previous item can get deleted
pub fn add(container: &mut Container, item_id: ItemId, location_id: LocationId) -> Result<()> {
    let is_money = container.objects.get_static_id(item_id) == container.config.money_id;

    if is_money {
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
    let item_id = match get_money_id(container, obj_id) {
        Some(item_id) => item_id,
        None => return Ok(Money(0)),
    };

    let item = container
        .items
        .get(item_id)
        .expect("mob money is not a item");
    Ok(item.amount.into())
}

pub fn get_money_id(container: &Container, obj_id: ObjId) -> Option<ItemId> {
    let money_static_id = container
        .config
        .money_id
        .expect("money_id is not define in configuration");

    container
        .locations
        .list_at(obj_id)
        .filter(|&id| container.objects.get_static_id(id) == Some(money_static_id))
        .next()
}

/// return the new money amount
pub fn remove_money(container: &mut Container, obj_id: ObjId, amount: Money) -> Result<Money> {
    let item_id = match get_money_id(container, obj_id) {
        Some(item_id) => item_id,
        None => return Err(Error::InvalidArgumentFailure),
    };

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

            debug!("{:?} receive {:?} money by merging inventory money", inventory_id, amount);

            Ok(())
        }
        (Some(item_id), None) => {
            let item = container
                .items
                .get_mut(item_id)
                .expect("Money was created but is not a item");
            item.amount += amount.as_u32();
            debug!("{:?} receive {:?} money by updating inventory money", inventory_id, amount);
            Ok(())
        }
        (None, Some(item_id)) => {
            container.locations.set(item_id, inventory_id);
            container.ownership.remove_owner(item_id);
            debug!("{:?} receive {:?} money by adding money to inventory", inventory_id, amount);
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
            debug!("{:?} receive {:?} money by creating money in inventory", inventory_id, amount);
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
    locations
        .list_at(obj_id)
        .flat_map(move |id| items.get(id))
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::loader::{ItemData, ItemFlagsData, ObjData, StaticId};
    use crate::game::obj::Obj;

    // not require, bug was foudn
    //    #[test]
    //    fn test_inventory_add() {
    //        let mut container = Container::new();
    //        let money_prefab = add_money_prefab(&mut container, 0);
    //        let inventory_id = add_inventory(&mut container);
    //        let money_1_id = spawn(&mut container, money_prefab);
    //        let money_2_id = spawn(&mut container, money_prefab);
    //        let money_3_id = spawn(&mut container, money_prefab);
    //
    //        super::add(&mut container, money_1_id, inventory_id).unwrap();
    //        check_money_amount(&container, inventory_id, 1);
    //
    //        super::add(&mut container, money_2_id, inventory_id).unwrap();
    //        check_money_amount(&container, inventory_id, 2);
    //
    //        super::add(&mut container, money_3_id, inventory_id).unwrap();
    //        check_money_amount(&container, inventory_id, 3);
    //    }
    //
    //    fn add_money_prefab(container: &mut Container, id: u32) -> StaticId {
    //        let mut item_flags_data = ItemFlagsData::new();
    //        item_flags_data.money = Some(true);
    //
    //        let mut item_data = ItemData::new();
    //        item_data.amount = Some(1);
    //        item_data.flags = Some(item_flags_data);
    //
    //
    //        let mut data = ObjData::new(StaticId(id));
    //        data.item = Some(item_data);
    //        container.loader.add_prefab(data);
    //
    //        StaticId(id)
    //    }
    //
    //    fn add_inventory(container: &mut Container) -> ObjId {
    //        unimplemented!()
    //    }
    //
    //    fn spawn(container: &mut Container, static_id: StaticId) -> ObjId {
    //        unimplemented!()
    //    }
    //
    //    fn check_money_amount(container: &Container, inventory_id: ObjId, amount: u32) -> ObjId {
    //        unimplemented!()
    //    }
}
