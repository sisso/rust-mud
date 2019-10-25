use socket_server::Server;
use crate::runner::Engine;

pub struct CommandLineController {

}

impl CommandLineController {
    pub fn new(server: Vec<Box<dyn Server>>) -> Self {
        CommandLineController {

        }
    }

    pub fn handle(&mut self, engine: &mut Engine) {

    }
}
