use crate::{Server, ServerOutput, ServerChanges, ConnectionId};

pub struct LocalServer {

}

impl LocalServer {
    pub fn new() -> Self {
        LocalServer {}
    }
}

impl Server for LocalServer {
    fn run(&mut self) -> ServerChanges {
        unimplemented!()
    }

    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        unimplemented!()
    }

    fn disconnect(&mut self, connection_id: ConnectionId) {
        unimplemented!()
    }
}
