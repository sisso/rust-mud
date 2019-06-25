use super::domain::*;
use super::game_controller::Output;
use super::command_handler;

pub struct HandleReturn {
    pub output: Option<Output>,
    pub command: Option<Command>
}

impl HandleReturn {
    fn output(output: Output) -> HandleReturn {
        HandleReturn{
            output: Some(output),
            command: None
        }
    }

    fn command(command: Command) -> HandleReturn {
        HandleReturn{
            output: None,
            command: Some(command)
        }
    }
}

// TODO: remove login?
pub fn handle(game: &mut Container, player_id: &PlayerId, input: String) -> HandleReturn {
    match input.as_ref() {
        "l" | "look" =>  handle_look(game, player_id),
        "n" | "s" | "e" | "w" => execute_move(game, player_id, &input),
        _ if input.starts_with("say ")  => {
            let msg = input["say ".len()..].to_string();
            execute_say(game, player_id, msg)
        },
        _ => HandleReturn::output(Output::private(*player_id, format!("unknown command '{}'", input))),
    }
}

pub fn handle_look(game: &Container, player_id: &PlayerId) -> HandleReturn {
    let look_output = command_handler::get_look_description(&game, &game.get_player_context(&player_id));
    HandleReturn::output(Output::private(*player_id, look_output))
}

fn execute_say(game: &Container, player_id: &PlayerId, msg: String) -> HandleReturn {
    HandleReturn::command(Command::Say {
        player_id: player_id.clone(),
        msg
    })
}

fn execute_move(game: &mut Container, player_id: &PlayerId, dir: &String) -> HandleReturn {
    let dir = match dir.as_ref() {
        "n" => Dir::N,
        "s" => Dir::S,
        "e" => Dir::E,
        "w" => Dir::W,
        _   => panic!("invalid input {}", dir),
    };

    HandleReturn::command(Command::Move {
        player_id: player_id.clone(),
        dir: dir
    })
}

fn resolve_avatar_id(game: &Container, player_id: &PlayerId) -> u32 {
    let player = game.get_player_by_id(player_id);
    player.avatar_id
}
