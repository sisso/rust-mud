use crate::errors::{AsResult, Error, Result};
use crate::game::actions_items::*;
use crate::game::container::Container;
use crate::game::item::{ItemId, ItemRepository};
use crate::game::labels::Labels;
use crate::game::location::Locations;
use crate::game::mob::MobId;
use crate::game::{comm, inventory};
use crate::utils::strinput::StrInput;
use commons::{ObjId, PlayerId};

#[derive(Debug)]
pub enum ParseItemError {
    ItemNotProvided,
    ItemNotFound { label: String },
}

pub fn parser_owned_item(
    container: &Container,
    owner_id: ObjId,
    args: StrInput,
) -> std::result::Result<ItemId, ParseItemError> {
    if args.plain_arguments().is_empty() {
        return Err(ParseItemError::ItemNotProvided);
    }

    let item_label = args.plain_arguments();

    let founds = inventory::search(
        &container.labels,
        &container.locations,
        &container.items,
        owner_id,
        item_label,
    );
    match founds.first().cloned() {
        Some(item_id) => Ok(item_id),
        None => Err(ParseItemError::ItemNotFound {
            label: item_label.to_string(),
        }),
    }
}

pub fn parse_not_owned_item(
    labels: &Labels,
    locations: &Locations,
    items: &ItemRepository,
    item_location: ObjId,
    args: StrInput,
) -> std::result::Result<(ItemId, Option<ItemId>), ParseItemError> {
    let is_preposition = { |s: &str| s.eq("in") || s.eq("at") || s.eq("from") };

    let args = args.parse_arguments();
    match (args.get(0), args.get(1), args.get(2)) {
        (Some(item_label), None, None) => {
            let found =
                inventory::search_one(&labels, &locations, &items, item_location, item_label)
                    .ok_or(ParseItemError::ItemNotFound {
                        label: item_label.to_string(),
                    })?;

            Ok((found, None))
        }
        (Some(item_label), Some(preposition), Some(container_label))
            if is_preposition(preposition) =>
        {
            let found_container =
                inventory::search_one(&labels, &locations, &items, item_location, container_label)
                    .ok_or(ParseItemError::ItemNotFound {
                        label: container_label.to_string(),
                    })?;

            let found_item =
                inventory::search_one(&labels, &locations, &items, found_container, item_label)
                    .ok_or(ParseItemError::ItemNotFound {
                        label: item_label.to_string(),
                    })?;

            Ok((found_item, Some(found_container)))
        }
        _ => Err(ParseItemError::ItemNotProvided),
    }
}

pub fn pickup(container: &mut Container, mob_id: MobId, args: StrInput) -> Result<()> {
    let room_id = container.locations.get(mob_id).as_result()?;

    match parse_not_owned_item(
        &container.labels,
        &container.locations,
        &container.items,
        room_id,
        args,
    ) {
        Ok((item_id, maybe_container)) => {
            let _ = do_pickup(container, mob_id, item_id, maybe_container);
        }
        Err(ParseItemError::ItemNotProvided) => {
            container.outputs.private(mob_id, comm::pick_what())
        }
        Err(ParseItemError::ItemNotFound { label }) => container
            .outputs
            .private(mob_id, comm::pick_where_not_found(label.as_str())),
    }

    Ok(())
}

pub fn equip(container: &mut Container, mob_id: MobId, args: StrInput) -> Result<()> {
    match parser_owned_item(&container, mob_id, args) {
        Ok(item_id) => do_equip(container, mob_id, item_id),
        Err(ParseItemError::ItemNotProvided) => {
            container.outputs.private(mob_id, comm::equip_what());
            Err(Error::InvalidArgumentFailure)
        }
        Err(ParseItemError::ItemNotFound { label }) => {
            container
                .outputs
                .private(mob_id, comm::equip_item_not_found(label.as_str()));
            Err(Error::InvalidArgumentFailure)
        }
    }
}

pub fn drop(container: &mut Container, mob_id: MobId, args: StrInput) -> Result<()> {
    parser_owned_item(&container, mob_id, args)
        .map_err(|err| {
            match err {
                ParseItemError::ItemNotProvided => container
                    .outputs
                    .private(mob_id, comm::drop_item_no_target()),
                ParseItemError::ItemNotFound { label } => container
                    .outputs
                    .private(mob_id, comm::drop_item_not_found(label.as_str())),
            };

            Error::InvalidArgumentFailure
        })
        .and_then(|item_id| do_drop(container, mob_id, item_id))
}

pub fn strip(container: &mut Container, mob_id: MobId, args: StrInput) -> Result<()> {
    parser_owned_item(&container, mob_id, args)
        .map_err(|err| {
            match err {
                ParseItemError::ItemNotProvided => {
                    container.outputs.private(mob_id, comm::strip_what())
                }
                ParseItemError::ItemNotFound { label } => container
                    .outputs
                    .private(mob_id, comm::strip_item_not_found(label.as_str())),
            };

            Error::InvalidArgumentFailure
        })
        .and_then(|item_id| do_strip(container, mob_id, item_id))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::game::actions_items::test::setup;

    #[test]
    fn test_parse_not_owned_item_not_found_in_room() {
        let scenery = setup();
        let result = parse_not_owned_item(
            &scenery.container.labels,
            &scenery.container.locations,
            &scenery.container.items,
            scenery.room_id,
            StrInput("get item3"),
        );
        match result {
            Err(ParseItemError::ItemNotFound { label }) => {
                assert_eq!(label.as_str(), "item3");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_not_owned_item_should_find_item_in_the_floor() {
        let scenery = setup();
        let result = parse_not_owned_item(
            &scenery.container.labels,
            &scenery.container.locations,
            &scenery.container.items,
            scenery.room_id,
            StrInput("get item1"),
        );
        match result {
            Ok((item_id, None)) => assert_eq!(item_id, scenery.item1_id),
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_not_owned_item_not_found_in_container() {
        let scenery = setup();
        let result = parse_not_owned_item(
            &scenery.container.labels,
            &scenery.container.locations,
            &scenery.container.items,
            scenery.room_id,
            StrInput("get item1 in container1"),
        );
        match result {
            Err(ParseItemError::ItemNotFound { label }) => {
                assert_eq!(label.as_str(), "item1");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_parse_not_owned_item_should_find_item_in_the_container() {
        let scenery = setup();
        let result = parse_not_owned_item(
            &scenery.container.labels,
            &scenery.container.locations,
            &scenery.container.items,
            scenery.room_id,
            StrInput("get item2 in container1"),
        );
        match result {
            Ok((item_id, Some(container_id))) => {
                assert_eq!(item_id, scenery.item2_id);
                assert_eq!(container_id, scenery.container_id);
            }
            _ => panic!(),
        }
    }
}
