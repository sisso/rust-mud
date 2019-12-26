use crate::game::mob::MobId;
use crate::errors::*;
use crate::game::container::{Container, Ctx};
use crate::utils::strinput::StrInput;
use commons::ObjId;
use crate::game::{Outputs, comm, inventory};
use crate::game::comm::VendorListItem;
use crate::game::prices::Money;
use crate::game::loader::Loader;

pub fn list(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, vendor_id: MobId) -> Result<()> {
    let list = container.prices.list()
        .map(|price| {
            let label = container.labels.get_label_f(price.id);
            VendorListItem { label, buy: price.buy, sell: price.sell }
        })
        .collect();

    let msg = comm::vendor_list(list);
    outputs.private(mob_id, msg);
    Ok(())
}

//pub fn buy(container: &Container, mob_id: MobId, target_id: ObjId) -> Result<()> {
//    unimplemented!();
//}
//

pub fn sell(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, item_id: ObjId) -> Result<()> {
    let sell_price = container.prices.get(item_id)
        .ok_or(Error::NotFound)
        .map_err(|err| {
            let label = container.labels.get_label_f(item_id);
            outputs.private(mob_id, comm::vendor_sell_item_not_found(label));
            err
        })?
        .sell;

    add_money(container, mob_id, sell_price)
        .map_err(|err| {
            outputs.private(mob_id, comm::vendor_operation_fail());
            err
        })?;

    container.remove(item_id);

    Ok(())
}

fn add_money(container: &mut Container, obj_id: ObjId, amount: Money) -> Result<()> {
    // check if already have any money item and append, otherwise find one in loader and spawn

    let item_id = inventory::get_inventory_list(&container.locations, &container.items, obj_id)
        .into_iter()
        .filter(|item| item.flags.is_money)
        .map(|item| item.id)
        .next();

    match item_id {
        Some(item_id) => {
            let item = container.items.get_mut(item_id).expect("Money was created but is not a item");
            item.amount += amount.as_u32();
            Ok(())
        },
        None => {
            let static_id = container.loader.find_money_static_id().expect("No money found");
            let item_id = Loader::spawn_at(container, static_id, obj_id)?;
            let item = container.items.get_mut(item_id).expect("Money was created but is not a item");
            item.amount = amount.as_u32();
            Ok(())
        }
    }
}
