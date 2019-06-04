use super::game::*;
use super::game_controller::HandleOutput;

struct PlayerCtx<'a> {
    player: &'a Player,
    avatar: &'a Mob,
    room: &'a Room,
}

// TODO: remove login?
pub fn handle(game: &mut Game, player_id: u32, login: &String, mut input: String) -> Vec<HandleOutput> {
    match input.as_ref() {
        "l" | "look" => out_private(player_id, handle_look(game, login)),
        "n" | "s" | "e" | "w" => execute_move(game, player_id, login, &input),
        _ if input.starts_with("say ")  => execute_say(game, player_id, login, &input.remove("say ".len()).to_string()),
        _ => out_private(player_id, format!("unknown command '{}'\n$ ", input)),
    }
}

pub fn handle_look(game: &Game, login: &String) -> String {
    let ctx = resolve_player(game, login);
    execute_look(game, &ctx)
}

fn out_private(id: u32, msg: String) -> Vec<HandleOutput> {
    vec![HandleOutput::private(id, msg)]
}

fn out_private_and_room(player_id: u32, player_msg: String, room_id: u32, room_msg: String) -> Vec<HandleOutput> {
    let o = HandleOutput {
        player_id:  player_id,
        player_msg: vec![player_msg],
        room_id:    Some(room_id),
        room_msg:   vec![room_msg]
    };

    vec![o]
}

fn execute_say(game: &Game, player_id: u32, login: &String, msg: &String) -> Vec<HandleOutput> {
    let ctx = resolve_player(game, login);
    let player_msg = format!("you say '{}'\n", msg);
    let room_msg = format!("{} says '{}'\n", ctx.avatar.label, msg);
    out_private_and_room(player_id, player_msg, ctx.room.id, room_msg)

}

fn execute_move(game: &mut Game, player_id: u32, login: &String, dir: &String) -> Vec<HandleOutput> {
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

            let player_msg = format!("you move to {}!\n\n{}", dir, look);
            let room_msg = format!("{} comes from {}.\n", ctx.avatar.label, dir.inv());

            out_private_and_room(player_id, player_msg, ctx.room.id, room_msg)
        },
        None => {
            out_private(player_id, format!("not possible to move to {}!\n\n$ ", dir))
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
