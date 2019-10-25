use core::utils::UserId;
use mud_engine::{Engine, Output};
use socket_server::{ServerOutput, ConnectionId};
use std::collections::HashMap;

pub fn handle_outputs(engine: &mut Engine, per_user_id: &HashMap<UserId, ConnectionId>, output: &Vec<Output>) -> Vec<ServerOutput> {
    unimplemented!()
}
