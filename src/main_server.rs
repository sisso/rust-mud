use crate::server_socket;
use crate::game::Game;
use crate::utils::*;

pub fn run() {
    let server = server_socket::SocketServer::new();
    let mut game = Game::new(Box::new(server), Some("/tmp/current.jsonp".to_string()));

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        game.run(Seconds(0.1));
    }
}
