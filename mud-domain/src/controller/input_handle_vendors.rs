use crate::controller::input_handle_items;
use crate::controller::input_handle_items::ParseItemError;
use crate::errors::*;
use crate::game::actions::out;
use crate::game::container::Container;
use crate::game::inventory;
use crate::game::item::ItemId;
use crate::game::loader::{dto::StaticId, Loader};
use crate::game::mob::MobId;
use crate::game::prices::Money;
use crate::game::{actions_vendor, comm, outputs::Outputs};
use crate::utils::strinput::StrInput;
use crate::utils::text;
use commons::ObjId;

pub fn list(container: &mut Container, mob_id: MobId) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, mob_id)?;
    actions_vendor::list(container, mob_id, vendor_id)
}

pub fn buy(container: &mut Container, mob_id: MobId, input: StrInput) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, mob_id)?;

    let plain_arguments = input.plain_arguments();
    if plain_arguments.is_empty() {
        return list(container, mob_id);
    }

    let static_id = match parse_vendor_item(container, vendor_id, plain_arguments) {
        Some(static_id) => static_id,
        None => {
            container
                .outputs
                .private(mob_id, comm::vendor_buy_item_not_found(plain_arguments));
            return Err(Error::InvalidArgumentFailure);
        }
    };

    actions_vendor::buy(container, mob_id, vendor_id, static_id).map(|_| ())
}

pub fn sell(container: &mut Container, mob_id: MobId, input: StrInput) -> Result<()> {
    let maybe_item = input_handle_items::parser_owned_item(container, mob_id, input);
    match maybe_item {
        Err(ParseItemError::ItemNotProvided) => list(container, mob_id),

        Err(ParseItemError::ItemNotFound { label }) => {
            container
                .outputs
                .private(mob_id, comm::vendor_sell_item_not_found(label.as_str()));
            Err(Error::InvalidArgumentFailure)
        }

        Ok(item_id) => actions_vendor::sell(container, mob_id, item_id),
    }
}

fn find_vendor_at_mob_location(container: &mut Container, mob_id: MobId) -> Result<ObjId> {
    let location_id = match container.locations.get(mob_id) {
        Some(location_id) => location_id,
        None => {
            container
                .outputs
                .private(mob_id, comm::vendor_operation_fail());
            return Err(Error::InvalidStateFailure);
        }
    };

    let option = container
        .locations
        .list_at(location_id)
        .into_iter()
        .filter(|&id| container.vendors.exist(id))
        .next();

    option.ok_or_else(|| {
        container
            .outputs
            .private(mob_id, comm::vendor_operation_fail());
        Error::InvalidArgumentFailure
    })
}

// TODO: move to a logic place
// TODO: is used in a single place, why is not defined there?
fn find_vendor_sell_item(
    mob_id: MobId,
    outputs: &mut Outputs,
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

pub fn parse_vendor_item(
    container: &Container,
    _vendor_id: ObjId,
    input: &str,
) -> Option<StaticId> {
    container
        .loader
        .list_prefabs()
        .filter(|data| data.price.is_some())
        .filter(|data| text::is_text_eq(data.label.as_str(), input))
        .map(|data| data.id)
        .next()
}
