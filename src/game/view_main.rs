use super::actions;
use super::comm;
use crate::game::domain::*;
use crate::game::controller::Output;

pub fn handle(container: &mut Container, outputs: &mut Vec<Output>, player_id: &PlayerId, input: String) {
    match input.as_ref() {
        "l" | "look" => {
            actions::look(container, outputs, player_id);
        },

        "n" | "s" | "e" | "w" => {
            let dir = match input.as_ref() {
                "n" => Dir::N,
                "s" => Dir::S,
                "e" => Dir::E,
                "w" => Dir::W,
                _ => panic!("invalid input {}", input),
            };

            actions::mv(container, outputs, player_id, dir)
        },

        "uptime" => {
            outputs.push(Output::private(player_id.clone(), comm::uptime(&container.get_time())));
        },

        _ if input.starts_with("say ")  => {
            let msg = input["say ".len()..].to_string();
            actions::say(container, outputs, player_id, msg);
        },

        _ => {
            outputs.push(Output::private(*player_id, comm::unknown_input(input)));
        },
    }
}
