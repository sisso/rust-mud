use crate::game::mob::MobId;
use crate::errors::*;
use crate::game::container::{Container, Ctx};
use crate::utils::strinput::StrInput;
use commons::ObjId;
use crate::game::{Outputs, comm};
use crate::game::comm::VendorListItem;

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
//pub fn sell(container: &Container, mob_id: MobId, target_id: ObjId) -> Result<()> {
//    unimplemented!();
//}
