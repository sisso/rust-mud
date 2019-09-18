use crate::server_socket;
use crate::utils::*;
use crate::game::server_runner::ServerRunner;

pub fn run() {
    let server = server_socket::SocketServer::new();
    let mut game = ServerRunner::new(Box::new(server), Some(("/tmp/current".to_string(), Second(1.0))));

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        game.run(Second(0.1));
    }
}
