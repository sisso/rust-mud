use super::game::*;

struct PlayerCtx<'a> {
    player: &'a Player,
    avatar: &'a Mob,
    room: &'a Room,
}

pub fn handle(game: &mut Game, login: &String, input: String) -> String {
    match input.as_ref() {
        "l" | "look" => handle_look(game, login),
        "n" | "s" | "e" | "w" => execute_move(game, login, &input),
        _ => format!("unknown command '{}'\n$ ", input),
    }
}

pub fn handle_look(game: &Game, login: &String) -> String {
    let ctx = resolve_player(game, login);
    execute_look(game, &ctx)
}


fn execute_move(game: &mut Game, login: &String, dir: &String) -> String {
    let dir = match dir.as_ref() {
        "n" => Dir::N,
        "s" => Dir::S,
        "e" => Dir::E,
        "w" => Dir::W,
        _   => panic!("invalid input {}", dir),
    };

    let ctx = resolve_player(game, login);

    let exit = ctx.room
        .exits
        .iter()
        .find(|e| e.0 == dir)
        .map(|i| i.clone());

    let avatar_id= ctx.avatar.id;

    match exit {
        Some(exit) => {
            // change entity in place
            let mob = game.get_mob_mut(avatar_id);
            mob.room_id = exit.1;

            let ctx = resolve_player(game, login);
            let look = execute_look(game, &ctx);

            format!("you move to {}!\n\n{}", dir, look)
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

fn resolve_player<'a, 'b>(game: &'a Game, login: &'b String) -> PlayerCtx<'a> {
    let player = game.get_player(&login);
    let mob    = game.get_mob(player.avatar_id);
    let room= game.get_room(mob.room_id);

    PlayerCtx {
        player,
        avatar: mob,
        room
    }
}
