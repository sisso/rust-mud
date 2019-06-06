use super::game::*;
use super::game_controller::HandleOutput;

struct PlayerCtx<'a> {
    player: &'a Player,
    avatar: &'a Mob,
    room: &'a Room,
}

// TODO: remove login?
pub fn handle(game: &mut Game, player_id: &PlayerId, login: &String, mut input: String) -> Vec<HandleOutput> {
    match input.as_ref() {
        "l" | "look" => out_private(player_id, handle_look(game, login)),
        "n" | "s" | "e" | "w" => execute_move(game, player_id, login, &input),
        _ if input.starts_with("say ")  => {
            let msg = &input["say ".len()..].to_string();
            execute_say(game, player_id, login, msg)
        },
        _ => out_private(player_id, format!("unknown command '{}'\n$ ", input)),
    }
}

pub fn handle_look(game: &Game, login: &String) -> String {
    let ctx = resolve_player(game, login);
    execute_look(game, &ctx)
}

fn out_private(id: &PlayerId, msg: String) -> Vec<HandleOutput> {
    vec![HandleOutput::private(*id, msg)]
}

fn out_private_and_room(player_id: PlayerId, player_msg: String, room_id: u32, room_msg: String) -> HandleOutput {
     HandleOutput {
        player_id:  player_id,
        player_msg: vec![player_msg],
        room_id:    Some(room_id),
        room_msg:   vec![room_msg]
    }
}

fn out_room(player_id: PlayerId, room_id: u32, room_msg: String) -> HandleOutput {
     HandleOutput {
        player_id:  player_id,
        player_msg: vec![],
        room_id:    Some(room_id),
        room_msg:   vec![room_msg]
    }
}

fn execute_say(game: &Game, player_id: &PlayerId, login: &String, msg: &String) -> Vec<HandleOutput> {
    let ctx = resolve_player(game, login);
    let player_msg = format!("you say '{}'\n", msg);
    let room_msg = format!("{} says '{}'\n", ctx.avatar.label, msg);
    vec![out_private_and_room(*player_id, player_msg, ctx.room.id, room_msg)]
}

fn execute_move(game: &mut Game, player_id: &PlayerId, login: &String, dir: &String) -> Vec<HandleOutput> {
    let dir = match dir.as_ref() {
        "n" => Dir::N,
        "s" => Dir::S,
        "e" => Dir::E,
        "w" => Dir::W,
        _   => panic!("invalid input {}", dir),
    };

    let ctx = resolve_player(game, login);

    let exit_room_id = ctx.room
        .exits
        .iter()
        .find(|e| e.0 == dir)
        .map(|i| i.1);

    let avatar_id= ctx.avatar.id;

    println!("{} {} {:?} {}", player_id, dir, exit_room_id, avatar_id);

    match exit_room_id {
        Some(exit_room_id) => {
            let previous_room_id = ctx.avatar.room_id;

            // change entity in place
            let mut mob = ctx.avatar.clone();
            mob.room_id = exit_room_id;
            game.update_mob(mob);

            let ctx = resolve_player(game, login);
            let look = execute_look(game, &ctx);

            let player_msg = format!("you move to {}!\n\n{}", dir, look);
            let enter_room_msg = format!("{} comes from {}.\n", ctx.avatar.label, dir.inv());
            let exit_room_msg = format!("{} goes to {}.\n", ctx.avatar.label, dir);

            vec![
                out_private_and_room(*player_id, player_msg, ctx.room.id, enter_room_msg),
                out_room(*player_id, previous_room_id, exit_room_msg)
            ]
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
    let room= game.get_room(&mob.room_id);

    PlayerCtx {
        player,
        avatar: mob,
        room
    }
}
