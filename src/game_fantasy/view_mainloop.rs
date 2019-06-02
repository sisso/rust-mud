use super::game::*;

pub fn handle(game: &Game, login: &String, input: String) -> String {
    match input.as_ref() {
        "l" | "look" => handle_look(game, login),
        _ => format!("unknown command '{}'\n$ ", input),
    }

}

pub fn handle_look(game: &Game, login: &String) -> String {
    let player = game.get_player(&login);
    let mob    = game.get_mob(player.avatar_id);
    let room= game.get_room(mob.room_id);

    format!("{}\n\n{}\n\n$ ", room.label, room.desc).to_string()
}
