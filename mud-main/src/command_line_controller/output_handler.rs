use core::utils::UserId;
use mud_engine::{Engine, Output};
use socket_server::{ServerOutput, ConnectionId};
use crate::command_line_controller::{Outputs};

pub fn handle_outputs(engine: &mut Engine, outputs: &mut dyn Outputs, events: &Vec<Output>) {
    unimplemented!()
}
