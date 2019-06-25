use super::controller::{Output, PlayerInputHandler};
use super::domain::*;
use super::command_handler;
use super::controller::view_mainloop;

pub struct DefaultPlayerInputHandler {

}

impl PlayerInputHandler for DefaultPlayerInputHandler {
    fn handle(&mut self, game: &mut Container, player_id: &PlayerId, outputs: &mut Vec<Output>, input: String) {
        let handle_return = view_mainloop::handle(game, &player_id, input);
        let (output, command) = (handle_return.output, handle_return.command);

        if let Some(out) = output {
            outputs.push(out);
        }

        if let Some(command) = command {
            command_handler::handle(game, outputs, command);
        }
    }
}
