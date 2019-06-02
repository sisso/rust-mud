use super::game::*;

struct PlayerCtx {
    player: Player,
    avatar: Mob,
    room: Room,
}

pub fn handle(game: &Game, login: &String, input: String) -> String {
    match input.as_ref() {
        "l" | "look" => handle_look(game, login),
        _ => format!("unknown command '{}'\n$ ", input),
    }
}

fn execute_look(game: &Game, ctx: &PlayerCtx) -> String {
    let mut exits = vec![];
    for exit in &ctx.room.exits {
        let dir = &exit.0;
        exits.push(dir.to_string());
    }
    let exits = exits.join(", ");
    format!("{}\n\n{}\n\n[{}]\n\n$ ", ctx.room.label, ctx.room.desc, exits).to_string()
}

fn resolve_player(game: &Game, login: &String) -> PlayerCtx {
    let player = game.get_player(&login);
    let mob    = game.get_mob(player.avatar_id);
    let room= game.get_room(mob.room_id);

    PlayerCtx {
        player,
        avatar: mob,
        room
    }
}

pub fn handle_look(game: &Game, login: &String) -> String {
    let ctx = resolve_player(game, login);
    execute_look(game, &ctx)
}
