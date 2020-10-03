use crate::controller::input_handle_items;
use crate::controller::input_handle_items::ParseItemError;
use crate::errors::*;
use crate::game::actions::out;
use crate::game::actions_vendor::VendorTradeObj;
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
        list(container, mob_id)
    } else {
        let trade = match parse_vendor_item(container, vendor_id, plain_arguments)? {
            Some(static_id) => static_id,
            None => {
                container
                    .outputs
                    .private(mob_id, comm::vendor_buy_item_not_found(plain_arguments));
                return Err(Error::InvalidArgumentFailure);
            }
        };

        actions_vendor::buy(container, mob_id, vendor_id, trade.static_id).map(|_| ())
    }
}

pub fn sell(container: &mut Container, mob_id: MobId, input: StrInput) -> Result<()> {
    let vendor_id = find_vendor_at_mob_location(container, mob_id)?;

    let maybe_item = input_handle_items::parser_owned_item(container, mob_id, input);
    match maybe_item {
        Err(ParseItemError::ItemNotProvided) => list(container, mob_id),

        Err(ParseItemError::ItemNotFound { label }) => {
            container
                .outputs
                .private(mob_id, comm::vendor_sell_item_not_found(label.as_str()));
            Err(Error::InvalidArgumentFailure)
        }

        Ok(item_id) => actions_vendor::sell(container, mob_id, item_id, vendor_id),
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

pub fn parse_vendor_item(
    container: &Container,
    vendor_id: ObjId,
    input: &str,
) -> Result<Option<VendorTradeObj>> {
    let list = actions_vendor::find_vendor_list(container, vendor_id)?;

    let found = list
        .into_iter()
        .filter(|trade| {
            container
                .loader
                .get_prefab(trade.static_id)
                .map(|data| {
                    data.label
                        .as_ref()
                        .map(|label| text::is_text_eq(label.as_str(), input))
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .next();

    Ok(found)
}

#[cfg(test)]
mod test {
    use crate::game::container::Container;

    fn setup_scenery(container: &mut Container) {
        // add 2 room
        // add 2 items
        // add 2 markets
        // add 2 vendors
        // add 1 mob
        // add money
        // buy from one
        // move to other oom
        // sell to other
    }

    #[test]
    fn test_buy_and_selling() {
        let mut container = Container::new();
        // @see item_system.test.test_decay

        setup_scenery(&mut container);
    }
}
