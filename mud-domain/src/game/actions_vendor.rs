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

    inventory::add_money(container, mob_id, sell_price)
        .map_err(|err| {
            outputs.private(mob_id, comm::vendor_operation_fail());
            err
        })?;

    // label must be collect before remove of item
    let item_label = container.labels.get_label_f(item_id).to_string();

    container.remove(item_id);

    let mob_label = container.labels.get_label_f(mob_id);
    let location_id = container.locations.get(mob_id).unwrap();
    outputs.private(mob_id, comm::vendor_sell_item(item_label.as_str(), sell_price));
    outputs.broadcast(Some(mob_id), location_id, comm::vendor_sell_item_for_others(mob_label, item_label.as_str()));

    Ok(())
}

