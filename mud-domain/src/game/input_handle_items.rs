use crate::game::container::Container;
use crate::game::{Outputs, comm};
use commons::PlayerId;
use crate::game::item::{ParseItemError, parser_item};
use crate::game::actions_items::{do_equip, do_drop};

pub fn equip(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {
    let player = container.players.get_player_by_id(player_id);
    let avatar_id = player.mob_id;
    match parser_item(&container.items, &container.locations, avatar_id, args) {
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
    match parser_item(&container.items, &container.locations, avatar_id, args) {
        Ok(item_id) => {
            let _ = do_drop(container, outputs, Some(player_id), avatar_id, item_id);
        },
        Err(ParseItemError::ItemNotProvided) => outputs.private(player_id, comm::drop_item_no_target()),
        Err(ParseItemError::ItemNotFound { label }) => outputs.private(player_id, comm::drop_item_not_found(label.as_str())),
    }
}

pub fn strip(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, args: Vec<String>) {

}
