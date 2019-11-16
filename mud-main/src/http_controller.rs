#[allow(warn)]

use http_server::HttpServer;
use mud_engine::{Engine, ConnectionEvent};

pub struct HttpController {

}

impl HttpController {
    pub fn new(_server: HttpServer) -> Self {
        HttpController {}
    }

    pub fn handle_inputs(&mut self, _engine: &mut Engine) {

    }

    pub fn handle_events(&mut self, _engine: &mut Engine, _events: &Vec<ConnectionEvent>) {
    }
}
