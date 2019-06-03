use super::game::*;

struct PlayerCtx {
    player: Player,
    avatar: Mob,
    room: Room,
}

pub fn handle(game: &mut Game, login: &String, input: String) -> String {
    let ctx = resolve_player(game, login);

    match input.as_ref() {
        "l" | "look" => execute_look(game, &ctx),
        "n" | "s" | "e" | "w" => execute_move(game, &ctx, &input),
        _ => format!("unknown command '{}'\n$ ", input),
    }
}

pub fn handle_look(game: &Game, login: &String) -> String {
    let ctx = resolve_player(game, login);
    execute_look(game, &ctx)
}


fn execute_move(game: &mut Game, ctx: &PlayerCtx, dir: &String) -> String {
    let dir = match dir.as_ref() {
        "n" => Dir::N,
        "s" => Dir::S,
        "e" => Dir::E,
        "w" => Dir::W,
        _   => panic!("invalid input {}", dir),
    };

    let exit = ctx.room
        .exits
        .iter()
        .find(|e| e.0 == dir);

    match exit {
        Some(exit) => {
            let mut mob = ctx.avatar.clone();
            mob.room_id = exit.1;
            game.update_mob(mob);

            let new_ctx = resolve_player(game, &ctx.player.login);
            let new_look = execute_look(game, &new_ctx);

            format!("you move to {}!\n\n{}", dir, new_look)
        },
        None => {
            format!("not possible to move to {}!\n\n$ ", dir)
        }
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
