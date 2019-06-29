use super::domain::*;
use super::mob::*;

pub fn add_player(game: &mut Container, login: &String) -> PlayerId {
    // add player avatar
    let mob_id = game.next_mob_id();

    let mut mob = Mob::new(
        mob_id,
        0,
        login.clone(),
        Attributes {
            attack: 12,
            defense: 12,
            damage: Damage {
                min: 1,
                max: 4,
                calm_down: Seconds(1.0)
            },
            pv: Pv {
                current: 10,
                max: 10
            }
        }
    );
    mob.is_avatar = true;

    game.add_mob(mob);

    // add player to game
    let player = game.player_connect(login.clone(), mob_id);
    player.id
}
