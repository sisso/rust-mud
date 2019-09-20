use crate::utils::{ConnectionId, ConnectionOutput};

pub mod server_dummy;
pub mod server_socket;

pub struct ServerChanges {
    pub connects: Vec<ConnectionId>,
    pub disconnects: Vec<ConnectionId>,
    pub pending_inputs: Vec<(ConnectionId, String)>,
}

pub trait Server {
    fn run(&mut self) -> ServerChanges;
    fn append_output(&mut self, pending_outputs: Vec<ConnectionOutput>);
}
