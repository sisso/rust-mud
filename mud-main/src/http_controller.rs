use http_server::HttpServer;
use mud_engine::{Engine, ConnectionEvent};

pub struct HttpController {

}

impl HttpController {
    pub fn new(server: HttpServer) -> Self {
        HttpController {}
    }

    pub fn handle_inputs(&mut self, engine: &mut Engine) {

    }

    pub fn handle_events(&mut self, engine: &mut Engine, events: &Vec<ConnectionEvent>) {
    }
}
