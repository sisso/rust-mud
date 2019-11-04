use commons::PlayerId;
use crate::game::container::Container;
use crate::game::{Outputs, player};

pub fn on_player_disconnect(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let player = container.players.get_player_by_id(player_id);
    let mob_id = player.mob_id;
    // TODO: move mob to limbo?
}

pub fn on_player_login(container: &mut Container, outputs: &mut dyn Outputs, login: &str) -> PlayerId {
    match container.players.login(login) {
        Some(player_id) => {
//            let avatar_id = container.players.get_player_by_id(player_id).avatar_id;
//            mob::respawn_avatar(container, outputs, avatar_id);
            player_id
        },
        None => {
            player::create_player(container, login)
        }
    }
}
