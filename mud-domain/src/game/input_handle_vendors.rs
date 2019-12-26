use crate::game::mob::MobId;
use crate::errors::*;
use crate::game::container::{Container, Ctx};
use crate::utils::strinput::StrInput;
use crate::game::{actions_vendor, Outputs, comm, input_handle_items};
use commons::ObjId;
use crate::game::prices::Money;
use crate::game::inventory;
use crate::game::loader::Loader;
use crate::game::input_handle_items::ParseItemError;
use crate::game::item::ItemId;
use crate::game::actions::out;

pub fn list(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, input: StrInput) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, outputs, mob_id)?;
    actions_vendor::list(container, outputs, mob_id, vendor_id)
}

pub fn buy(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, input: StrInput) -> Result<()> {
    unimplemented!();
}

pub fn sell(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId, input: StrInput) -> Result<()> {
    let _ = find_vendor_at_mob_location(container, outputs, mob_id)?;
    let item_id = find_vendor_item(outputs, mob_id,input_handle_items::parser_owned_item(container, mob_id, input.split()))?;

    actions_vendor::sell(container, outputs, mob_id, item_id)
}

fn find_vendor_at_mob_location(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<ObjId> {
    let location_id = match container.locations.get(mob_id) {
        Some(location_id) => location_id,
        None => {
            outputs.private(mob_id, comm::vendor_operation_fail());
            return Err(Error::IllegalState);
        },
    };

    container.locations.list_at(location_id)
        .into_iter()
        .filter(|&id| container.vendors.exist(id))
        .next()
        .ok_or_else(|| {
            outputs.private(mob_id, comm::vendor_operation_fail());
            Error::NotFound
        })
}

// TODO: move to a logic place
fn find_vendor_item(outputs: &mut dyn Outputs, mob_id: MobId, result: std::result::Result<ItemId, ParseItemError>) -> Result<ItemId> {
    result.map_err(|err| {
        match err {
            ParseItemError::ItemNotFound { label } => {
                outputs.private(mob_id, comm::vendor_sell_item_not_found(label.as_str()));
                Error::NotFound
            }
            ParseItemError::ItemNotProvided => {
                outputs.private(mob_id, comm::vendor_sell_item_not_found(""));
                Error::NotFound
            }
        }
    })
}

