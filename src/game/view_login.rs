use crate::game::domain::{PlayerId, Container};
use crate::game::players;

pub struct LoginResult {
    pub player_id: Option<PlayerId>,
    pub msg: String,
}

pub fn handle(game: &mut Container, input: String) -> LoginResult {
    if input.len() > 3 {
        let login = input;
        let player_id = players::add_player(game, &login);
        let msg = format!("login success, welcome {}\n\n", login);
        LoginResult {
            player_id: Some(player_id),
            msg,
        }
    } else {
        LoginResult {
            player_id: None,
            msg: format!("invalid login {}\n\nlogin: ", input),
        }
    }
}

pub fn handle_welcome() -> String {
    "Welcome to MUD\n--------------\n\nlogin: ".to_string()
}
