use crate::game::container::Container;
use crate::game::{Outputs, comm, inventory};
use commons::{PlayerId, ObjId};
use crate::game::item::{ItemRepository, ItemId};
use crate::game::actions_items::{do_equip, do_drop, do_strip, do_pickup};
use crate::game::location::Locations;

#[derive(Debug)]
pub enum ParseItemError {
    ItemNotProvided,
    ItemNotFound { label: String },
}

pub fn parser_owned_item(items: &ItemRepository, locations: &Locations, item_location: ObjId, args: Vec<String>) -> Result<ItemId, ParseItemError> {
    let item_label = match args.get(1) {
        Some(str) => str,
        None => return Err(ParseItemError::ItemNotProvided),
    };

    let founds = inventory::search(&locations, &items, item_location, item_label.as_str());
    match founds.first() {
        Some(item) => Ok(item.id),
        None => Err(ParseItemError::ItemNotFound { label: item_label.to_string() }),
    }
}

/// formats
///
/// get object
/// get object in|at|from container
pub fn parse_not_owned_item(items: &ItemRepository,
                            locations: &Locations,
                            item_location: ObjId,
                            args: Vec<String>) -> Result<(ItemId, Option<ItemId>), ParseItemError> {

    let is_in = {|s: &String|
        s.as_str().eq("in") ||
        s.as_str().eq("at") ||
        s.as_str().eq("from")
    };

    match (args.get(1), args.get(2), args.get(3)) {
        (Some(item_label), None, None) => {
            let found = inventory::search_one(&locations, &items, item_location, item_label.as_str())
                .ok_or(ParseItemError::ItemNotFound { label: item_label.clone() })?;

            Ok((found.id, None))
        }
        (Some(item_label), Some(preposition), Some(container_label)) if is_in(preposition) => {
            let found_container = inventory::search_one(&locations, &items, item_location, container_label.as_str())
                .ok_or(ParseItemError::ItemNotFound { label: container_label.clone() })?;

            let found_item = inventory::search_one(&locations, &items, found_container.id, item_label)
                .ok_or(ParseItemError::ItemNotFound { label: item_label.clone() })?;

            Ok((found_item.id, Some(found_container.id)))
        }
        _ => { Err(ParseItemError::ItemNotProvided) },
    }
}

pub fn pickup(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) -> Result<(),()> {
    let player = container.players.get_player_by_id(player_id);
    let mob_id = player.mob_id;
    let room_id = container.locations.get(mob_id)?;

    match parse_not_owned_item(&container.items, &container.locations, room_id, args) {
        Ok((item_id, maybe_container)) => {
            let _ = do_pickup(container, outputs, Some(player_id), mob_id, item_id, maybe_container);
        },
        Err(ParseItemError::ItemNotProvided) => outputs.private(player_id, comm::pick_what()),
        Err(ParseItemError::ItemNotFound { label }) => outputs.private(player_id, comm::pick_where_not_found(label.as_str())),
    }

    Ok(())
}

pub fn equip(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    let player = container.players.get_player_by_id(player_id);
    let avatar_id = player.mob_id;
    match parser_owned_item(&container.items, &container.locations, avatar_id, args) {
        Ok(item_id) => {
            let _ = do_equip(container, outputs, Some(player_id),avatar_id, item_id);
        },
        Err(ParseItemError::ItemNotProvided) => outputs.private(player_id, comm::equip_what()),
        Err(ParseItemError::ItemNotFound { label }) => outputs.private(player_id, comm::equip_item_not_found(label.as_str())),
    }
}

pub fn drop(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    let player = container.players.get_player_by_id(player_id);
    let avatar_id = player.mob_id;
    match parser_owned_item(&container.items, &container.locations, avatar_id, args) {
        Ok(item_id) => {
            let _ = do_drop(container, outputs, Some(player_id), avatar_id, item_id);
        },
        Err(ParseItemError::ItemNotProvided) => outputs.private(player_id, comm::drop_item_no_target()),
        Err(ParseItemError::ItemNotFound { label }) => outputs.private(player_id, comm::drop_item_not_found(label.as_str())),
    }
}

pub fn strip(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    let player = container.players.get_player_by_id(player_id);
    let avatar_id = player.mob_id;
    match parser_owned_item(&container.items, &container.locations, avatar_id, args) {
        Ok(item_id) => {
            let _ = do_strip(container, outputs, Some(player_id), avatar_id, item_id);
        },
        Err(ParseItemError::ItemNotProvided) => outputs.private(player_id, comm::strip_what()),
        Err(ParseItemError::ItemNotFound { label }) => outputs.private(player_id, comm::strip_item_not_found(label.as_str())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::builder;
    use crate::game::container::Container;
    use crate::game::OutputsBuffer;
    use crate::game::room::RoomId;

    struct TestScenery {
        outputs: OutputsBuffer,
        container: Container,
        room_id: RoomId,
        container_id: ItemId,
        item1_id: ItemId,
        item2_id: ItemId,
    }

    #[test]
    fn test_parse_not_owned_item_not_found_in_room() {
        let scenery= setup();
        let result = parse_not_owned_item(&scenery.container.items, &scenery.container.locations, scenery.room_id, vec!["get".to_string(), "item3".to_string()]);
        match result {
            Err(ParseItemError::ItemNotFound { label }) => {
                assert_eq!(label.as_str(), "item3");
            },
            _ => panic!()
        }
    }

    #[test]
    fn test_parse_not_owned_item_should_find_item_in_the_floor() {
        let scenery= setup();
        let result = parse_not_owned_item(&scenery.container.items, &scenery.container.locations, scenery.room_id, vec!["get".to_string(), "item1".to_string()]);
        match result {
            Ok((item_id, None)) => assert_eq!(item_id, scenery.item1_id),
            _ => panic!()
        }
    }

    #[test]
    fn test_parse_not_owned_item_not_found_in_container() {
        let scenery= setup();
        let result = parse_not_owned_item(&scenery.container.items, &scenery.container.locations, scenery.room_id, vec!["get".to_string(), "item1".to_string(), "in".to_string(), "container1".to_string()]);
        match result {
            Err(ParseItemError::ItemNotFound { label }) => {
              assert_eq!(label.as_str(), "item1");
            },
            _ => panic!()
        }
    }

    #[test]
    fn test_parse_not_owned_item_should_find_item_in_the_container() {
        let scenery= setup();
        let result = parse_not_owned_item(&scenery.container.items, &scenery.container.locations, scenery.room_id, vec!["get".to_string(), "item2".to_string(), "in".to_string(), "container1".to_string()]);
        match result {
            Ok((item_id, Some(container_id))) => {
                assert_eq!(item_id, scenery.item2_id);
                assert_eq!(container_id, scenery.container_id);
            },
            _ => panic!()
        }
    }

    fn setup() -> TestScenery {
        let mut outputs = OutputsBuffer::new();
        let mut container = Container::new();
        let room_id = builder::add_room(&mut container, "test_room", "");
        let container_id = builder::add_item(&mut container, "container1", room_id);
        let item1_id = builder::add_item(&mut container, "item1", room_id);
        let item2_id = builder::add_item(&mut container, "item2", container_id);

        TestScenery {
            outputs,
            container,
            room_id,
            container_id,
            item1_id,
            item2_id
        }
    }
}
