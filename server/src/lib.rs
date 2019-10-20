extern crate commons;

pub mod server_dummy;
pub mod server_socket;

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct ServerConnectionId(pub u32);

#[derive(Debug,Clone)]
pub struct ServerConnectionOutput {
    pub dest_connections_id: Vec<ServerConnectionId>,
    pub output: String
}

#[derive(Debug,Clone)]
pub struct ServerChanges {
    pub connects: Vec<ServerConnectionId>,
    pub disconnects: Vec<ServerConnectionId>,
    pub pending_inputs: Vec<(ServerConnectionId, String)>,
}

pub trait Server {
    fn run(&mut self) -> ServerChanges;
    fn append_output(&mut self, pending_outputs: Vec<ServerConnectionOutput>);
}
