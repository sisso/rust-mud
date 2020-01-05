use crate::errors::*;
use crate::game::actions::out;
use crate::game::container::{Container};
use crate::game::inventory;
use crate::game::item::ItemId;
use crate::game::loader::{Loader, StaticId};
use crate::game::mob::MobId;
use crate::game::prices::Money;
use crate::game::{actions_vendor, comm, Outputs};
use crate::utils::strinput::StrInput;
use crate::utils::text;
use commons::ObjId;
use crate::controller::input_handle_items::ParseItemError;
use crate::controller::input_handle_items;

pub fn list(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: StrInput,
) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, outputs, mob_id)?;
    actions_vendor::list(container, outputs, mob_id, vendor_id)
}

pub fn buy(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: StrInput,
) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, outputs, mob_id)?;

    let plain_arguments = input.plain_arguments();
    if plain_arguments.is_empty() {
        outputs.private(mob_id, comm::vendor_buy_item_not_found(plain_arguments));
        return Err(Error::InvalidArgumentFailure);
    }

    let static_id = match parse_vendor_item(container, vendor_id, plain_arguments) {
        Some(static_id) => static_id,
        None => {
            outputs.private(mob_id, comm::vendor_buy_item_not_found(plain_arguments));
            return Err(Error::InvalidArgumentFailure);
        }
    };

    actions_vendor::buy(container, outputs, mob_id, vendor_id, static_id).map(|_| ())
}

pub fn sell(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: StrInput,
) -> Result<()> {
    let _ = find_vendor_at_mob_location(container, outputs, mob_id)?;
    let item = input_handle_items::parser_owned_item(container, mob_id, input);
    let item_id = find_vendor_sell_item(outputs, mob_id, item)?;

    actions_vendor::sell(container, outputs, mob_id, item_id)
}

fn find_vendor_at_mob_location(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<ObjId> {
    let location_id = match container.locations.get(mob_id) {
        Some(location_id) => location_id,
        None => {
            outputs.private(mob_id, comm::vendor_operation_fail());
            return Err(Error::InvalidStateFailure);
        }
    };

    container
        .locations
        .list_at(location_id)
        .into_iter()
        .filter(|&id| container.vendors.exist(id))
        .next()
        .ok_or_else(|| {
            outputs.private(mob_id, comm::vendor_operation_fail());
            Error::InvalidArgumentFailure
        })
}

// TODO: move to a logic place
// TODO: is used in a single place, why is not defined there?
fn find_vendor_sell_item(
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    result: std::result::Result<ItemId, ParseItemError>,
) -> Result<ItemId> {
    result.map_err(|err| match err {
        ParseItemError::ItemNotFound { label } => {
            outputs.private(mob_id, comm::vendor_sell_item_not_found(label.as_str()));
            Error::InvalidArgumentFailure
        }
        ParseItemError::ItemNotProvided => {
            outputs.private(mob_id, comm::vendor_sell_item_not_found(""));
            Error::InvalidArgumentFailure
        }
    })
}

pub fn parse_vendor_item(container: &Container, vendor_id: ObjId, input: &str) -> Option<StaticId> {
    container
        .loader
        .list_prefabs()
        .filter(|data| data.price.is_some())
        .filter(|data| text::is_valid_search(data.label.as_str(), input))
        .map(|data| data.id)
        .next()
}
