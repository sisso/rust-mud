use http_server::HttpServer;
use crate::runner::Engine;

pub struct HttpController {

}

impl HttpController {
    pub fn new(server: HttpServer) -> Self {
        HttpController {}
    }

    pub fn handle(&mut self, engine: &mut Engine) {

    }
}
