use super::comm;
use super::domain::*;
use super::controller::Output;

pub enum Command {
    Move {
        player_id: PlayerId,
        dir: Dir
    },
    Say {
        player_id: PlayerId,
        msg: String
    }
}

impl Command {
    pub fn get_player_id(&self) -> &PlayerId {
        match self {
            Command::Move { player_id, ..} => player_id,
            Command::Say { player_id, ..} => player_id,
        }
    }
}

pub fn look(game: &mut Container, outputs: &mut Vec<Output>, player_id: &PlayerId) {
    let ctx = game.get_player_context(&player_id);

    outputs.push(Output::Private {
        player_id: player_id.clone(),
        msg: comm::get_look_description(game, &ctx)
    })
}

pub fn say(game: &mut Container, outputs: &mut Vec<Output>, player_id: &PlayerId, msg: String) {
    handle(game, outputs, Command::Say {
        player_id: player_id.clone(),
        msg: msg
    });
}

pub fn mv(game: &mut Container, outputs: &mut Vec<Output>, player_id: &PlayerId, dir: Dir) {
    handle(game, outputs, Command::Move {
        player_id: player_id.clone(),
        dir: dir
    });
}


pub fn handle(game: &mut Container, outputs: &mut Vec<Output>, command: Command) {
    let player_id = command.get_player_id().clone();
    let ctx = game.get_player_context(&player_id);

    match command {
        Command::Move { player_id, dir } => {
            let exit_room_id = ctx.room
                .exits
                .iter()
                .find(|e| e.0 == dir)
                .map(|i| i.1);

            match exit_room_id {
                Some(exit_room_id) => {
                    let previous_room_id = ctx.avatar.room_id;

                    // change entity in place
                    let mut mob = ctx.avatar.clone();
                    mob.room_id = exit_room_id;
                    game.update_mob(mob);

                    // get new player ctx
                    let ctx = game.get_player_context(&player_id);

                    let look = comm::get_look_description(&game, &ctx);

                    let player_msg = format!("you move to {}!\n\n{}", dir, look);
                    let enter_room_msg = format!("{} comes from {}.\n", ctx.avatar.label, dir.inv());
                    let exit_room_msg = format!("{} goes to {}.\n", ctx.avatar.label, dir);

                    outputs.push(Output::private(player_id, player_msg));
                    outputs.push(Output::room(player_id, previous_room_id, exit_room_msg));
                    outputs.push(Output::room(player_id, ctx.room.id, enter_room_msg));
                },
                None => {
                    outputs.push(Output::private(player_id, format!("not possible to move to {}!\n", dir)));
                }
            }
        },
        Command::Say { player_id, msg } => {
            let player_msg = format!("you say '{}'\n", msg);
            let room_msg = format!("{} says '{}'\n", ctx.avatar.label, msg);

            outputs.push(Output::private(player_id.clone(), player_msg));
            outputs.push(Output::room(player_id.clone(), ctx.avatar.room_id, room_msg));
        },
    }
}
