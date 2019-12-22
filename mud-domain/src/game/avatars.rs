use crate::game::container::Container;
use crate::game::labels::Label;
use crate::game::location::LocationId;
use crate::game::mob::{Attributes, Damage, Mob, MobId, Pv};
use crate::game::{comm, Outputs};
use commons::{DeltaTime, PlayerId};
use crate::game::player::Player;
use crate::errors::{Error, Result};
use crate::game::loader::Loader;

pub fn on_player_disconnect(
    container: &mut Container,
    _outputs: &mut dyn Outputs,
    player_id: PlayerId,
) {
    let player = container.players.get(player_id);
    let _mob_id = player.mob_id;
}

pub fn on_player_login(
    container: &mut Container,
    _outputs: &mut dyn Outputs,
    login: &str,
) -> PlayerId {
    match container.players.login(login) {
        Some(player_id) => player_id,
        None => create_player(container, login),
    }
}

// TODO: use trigger
pub fn respawn_avatar(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<()> {
    container.mobs.update(mob_id, |mob| {
        assert!(mob.is_avatar);
        mob.attributes.pv.current = 1;
    })?;
    let room_id = container.config.initial_room.unwrap();

    let mob_label = container.labels.get_label(mob_id).unwrap();

    container.locations.set(mob_id, room_id);

    outputs.private(mob_id, comm::mob_you_resurrected());
    outputs.broadcast(Some(mob_id), room_id, comm::mob_resurrected(mob_label));

    Ok(())
}

pub fn create_player(container: &mut Container, login: &str) -> PlayerId {
    let avatar_static_id = container.config.avatar_id.unwrap();
    let room_id = container.config.initial_room.unwrap();
    let player_id = container.objects.create();

    Loader::spawn_at(container, avatar_static_id, room_id);

    let mob_id = container.objects.create();


    let mut mob = Mob::new(mob_id);
    mob.is_avatar = true;
    mob.attributes = Attributes {
        attack: 12,
        defense: 12,
        damage: Damage { min: 1, max: 4 },
        pv: Pv {
            current: 10,
            max: 10,
            heal_rate: DeltaTime(1.0),
        },
        attack_calm_down: DeltaTime(1.0),
    };
    container.mobs.add(mob);

    container.locations.set(mob_id, room_id);
    container.labels.set(Label {
        id: mob_id,
        label: login.to_string(),
        code: login.to_string(),
        desc: login.to_string(),
    });

    // add player to game
    let player = container
        .players
        .create(player_id, login.to_string(), mob_id);
    player.id
}

pub fn find_deep_all_players_in(container: &Container, location_id: LocationId) -> Vec<PlayerId> {
    let candidates = container.locations.list_deep_at(location_id);
    candidates
        .into_iter()
        .flat_map(|id| container.players.find_from_mob(id))
        .collect()
}

