use crate::game::avatars;
use crate::game::container::Container;
use crate::game::triggers::EventKind;
use logs::*;

pub fn start(container: &mut Container) {}

pub fn run(container: &mut Container) {
    let mut players_to_respawn = vec![];

    for event in container.triggers.list(EventKind::Killed) {
        container
            .players
            .list()
            .find(|i| i.mob_id == event.get_obj_id())
            .iter()
            .for_each(|player| {
                players_to_respawn.push(player.id);
            });
    }

    for player_id in players_to_respawn {
        let _ = avatars::respawn_avatar(container, player_id).map_err(|err| {
            warn!("fail to respawn avatar for player {:?}", player_id);
        });
    }
}
