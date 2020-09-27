use crate::errors::{Error, Result};
use crate::game::container::Container;
use crate::game::labels::Label;
use crate::game::loader::Loader;
use crate::game::location::LocationId;
use crate::game::mob::{Attributes, Damage, Mob, MobId, Pv};
use crate::game::player::Player;
use crate::game::{comm, outputs::Outputs};
use commons::{DeltaTime, PlayerId};

pub fn on_player_disconnect(_container: &mut Container, _player_id: PlayerId) {
    // let player = container.players.get(player_id).as_result()?;
    // let _mob_id = player.mob_id;
}

pub fn on_player_login(container: &mut Container, login: &str) -> Result<PlayerId> {
    match container.players.login(login) {
        Some(player_id) => Ok(player_id),
        None => create_player(container, login),
    }
}

// TODO: use trigger
pub fn respawn_avatar(container: &mut Container, mob_id: MobId) -> Result<()> {
    container.mobs.update(mob_id, |mob| {
        assert!(mob.is_avatar);
        mob.attributes.pv.current = 1;
    })?;
    let room_id = container.config.initial_room.unwrap();

    let mob_label = container.labels.get_label(mob_id).unwrap();

    container.locations.set(mob_id, room_id);

    container
        .outputs
        .private(mob_id, comm::mob_you_resurrected());
    container
        .outputs
        .broadcast(Some(mob_id), room_id, comm::mob_resurrected(mob_label));

    Ok(())
}

pub fn create_player(container: &mut Container, login: &str) -> Result<PlayerId> {
    let avatar_static_id = container.config.avatar_id.unwrap();
    let room_id = container.config.initial_room.unwrap();
    let player_id = container.objects.create();

    let mob_id = Loader::spawn_at(container, avatar_static_id, room_id)?;

    container.labels.update(Label::new(mob_id, login));

    container
        .mobs
        .update(mob_id, |mob| mob.is_avatar = true)
        .unwrap();

    // add avatar location to memories
    container.memories.add(mob_id, room_id).unwrap();

    // add player to game
    let player = container
        .players
        .create(player_id, login.to_string(), mob_id);

    container.labels.update(Label {
        id: player_id,
        label: login.to_string(),
        code: login.to_string(),
        desc: format!("Player {}", login),
    });

    Ok(player.id)
}

pub fn find_deep_players_in(container: &Container, location_id: LocationId) -> Vec<PlayerId> {
    let candidates = container.locations.list_deep_at(location_id);

    candidates
        .into_iter()
        .flat_map(|id| container.players.find_from_mob(id))
        .collect()
}

pub fn find_players_in(container: &Container, location_id: LocationId) -> Vec<PlayerId> {
    let candidates = container.locations.list_at(location_id);

    candidates
        .flat_map(|id| container.players.find_from_mob(id))
        .collect()
}
