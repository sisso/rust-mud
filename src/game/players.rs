use super::domain::*;

pub fn add_player(game: &mut Container, login: &String) -> PlayerId {
    // add player avatar
    let mob_id = game.next_mob_id();

    let mob = Mob {
        id: mob_id,
        label: login.clone(),
        room_id: 0,
        is_avatar: true
    };

    game.add_mob(mob);

    // add player to game
    let player = game.player_connect(login.clone(), mob_id);
    player.id
}
