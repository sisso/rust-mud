use commons::{PlayerId, DeltaTime};
use crate::game::container::Container;
use crate::game::{Outputs, comm};
use crate::game::mob::{Mob, Attributes, Damage, Pv, MobId};
use crate::game::labels::Label;
use crate::game::location::LocationId;

//struct PlayerC<'a> {
//    pub coniner: &'a Container,
//    pub player_id: PlayerId,
//    player: Option<&'a Player>,
//    mob_id: MobId,
//    mob: Option<&'a Mob>,
//    room_id: Option<Room_id>,
//}
//
//impl PlayerC {
//    pub fn get_mob_id(&mut self) -> MobId {
//        self.conatiner.players.get_player_by_id(self.player_id);
//    }
//}

pub fn on_player_disconnect(container: &mut Container, _outputs: &mut dyn Outputs, player_id: PlayerId) {
    let player = container.players.get(player_id);
    let _mob_id = player.mob_id;
}

pub fn on_player_login(container: &mut Container, _outputs: &mut dyn Outputs, login: &str) -> PlayerId {
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

    let player_id = container.players.find_from_mob(mob.id).unwrap();
    let mob_label = container.labels.get_label(mob_id).unwrap();

    outputs.private(player_id, comm::mob_you_resurrected());
    outputs.room(player_id, room_id, comm::mob_resurrected(mob_label));

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

pub fn find_deep_all_players_in(container: &Container, location_id: LocationId) -> Vec<PlayerId> {
    let candidates = container.locations.list_deep_at(location_id);
    candidates.into_iter().flat_map(|id| {
       container.players.find_from_mob(id).ok()
    }).collect()
}
