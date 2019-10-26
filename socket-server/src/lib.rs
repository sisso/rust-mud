extern crate logs;

pub mod server_dummy;
pub mod server_socket;
pub mod local_server;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct ConnectionId(pub u32);

impl ConnectionId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// Going outside the server, to the user
#[derive(Debug,Clone)]
pub struct ServerOutput {
    pub connection_id: ConnectionId,
    pub msg: String
}

/// Coming from user into the server
#[derive(Debug,Clone)]
pub struct ServerInput {
    pub connection_id: ConnectionId,
    pub msg: String
}

#[derive(Debug,Clone)]
pub struct ServerChanges {
    pub connects: Vec<ConnectionId>,
    pub disconnects: Vec<ConnectionId>,
    pub inputs: Vec<ServerInput>,
}

pub trait Server {
    fn run(&mut self) -> ServerChanges;
    fn output(&mut self, connection_id: ConnectionId, msg: String);
    fn disconnect(&mut self, connection_id: ConnectionId);
}
