use crate::{Server, ServerOutput, ServerChanges};

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

    fn append_output(&mut self, outputs: Vec<ServerOutput>) {
        unimplemented!()
    }
}
