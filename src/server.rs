#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct ConnectionId {
    pub id: u32
}

impl ConnectionId {
    pub fn new(id: u32) -> Self {
        ConnectionId {
            id
        }
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ConnectionId({})", self.id)
    }
}

pub struct LoopResult {
    pub connects: Vec<ConnectionId>,
    pub disconnects: Vec<ConnectionId>,
    pub pending_inputs: Vec<(ConnectionId, String)>,
}

#[derive(Debug)]
pub struct Output {
    pub dest_connections_id: Vec<ConnectionId>,
    pub output: String
}

pub trait Server {
    fn run(&mut self, pending_outputs: Vec<Output>) -> LoopResult;
}
