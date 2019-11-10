use commons::{PlayerId, ObjId, DeltaTime};
use crate::game::container::Container;
use crate::game::{Outputs, player, comm, avatars};
use crate::game::room::RoomId;
use crate::game::mob::{Mob, Attributes, Damage, Pv, MobId};
use crate::game::labels::Label;

pub fn on_player_disconnect(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let player = container.players.get_player_by_id(player_id);
    let mob_id = player.mob_id;
}

pub fn on_player_login(container: &mut Container, outputs: &mut dyn Outputs, login: &str) -> PlayerId {
    match container.players.login(login) {
        Some(player_id) => {
            player_id
        },
        None => {
            create_player(container, login)
        }
    }
}

// TODO: use trigger
pub fn respawn_avatar(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<(),()> {
    let mut mob = container.mobs.get(mob_id)?.clone();
    assert!(mob.is_avatar);

    let room_id = container.config.initial_room;

    mob.attributes.pv.current = 1;

    let player = container.players.find_player_from_avatar_mob_id(mob.id);
    let player = player.unwrap();

    let mob_label = container.labels.get_label(mob_id).unwrap();

    outputs.private(player.id, comm::mob_you_resurrected());
    outputs.room(player.id, room_id, comm::mob_resurrected(mob_label));

    container.mobs.update(mob);
    container.locations.set(mob_id, room_id);
    Ok(())
}

pub fn create_player(container: &mut Container, login: &str) -> PlayerId {
    let room_id = container.config.initial_room;
    let player_id= container.objects.create();
    let mob_id = container.objects.create();

    let mut mob = Mob::new(
        mob_id,
    );
    mob.is_avatar = true;
    mob.attributes = Attributes {
        attack: 12,
        defense: 12,
        damage: Damage {
            min: 1,
            max: 4,
        },
        pv: Pv {
            current: 10,
            max: 10,
            heal_rate: DeltaTime(1.0),
        },
        attack_calm_down: DeltaTime(1.0)
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
    let player = container.players.create(player_id, login.to_string(), mob_id);
    player.id
}

