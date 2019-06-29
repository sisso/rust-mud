use super::domain::*;
use super::mob::*;

pub fn add_player(game: &mut Container, login: &String) -> PlayerId {
    // add player avatar
    let mob_id = game.next_mob_id();

    let mut mob = Mob::new(
        mob_id,
        0,
        login.clone()
    );
    mob.is_avatar = true;

    game.add_mob(mob);

    // add player to game
    let player = game.player_connect(login.clone(), mob_id);
    player.id
}
