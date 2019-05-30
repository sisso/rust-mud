use std::thread;
use std::net::TcpListener;

use crate::player::Player;
use crate::player_connection::PlayerConnection;

use crate::view_login;

pub struct Server {
    players: Vec<Player>
}

impl Server {
    pub fn new() -> Self {
        Server {
            players: Vec::new()
        }
    }

    // https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
    pub fn run(&self) {
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        println!("Server listening on port 3333");

        loop {
            if let Ok((mut stream, addr)) = listener.accept() {
                println!("New connection: {}", addr);

                thread::spawn(move || {
                    // connection succeeded
                    let connection = PlayerConnection::new(stream);
                    let player = view_login::handle_login(connection)
                        .expect("failed to handle connection login");
                    let addr = player.connection.addr().expect("failed to get connection address");
                    println!("Login complete for {}, user is '{}'", addr, player.name);
//                        players.push(player);
                });
            }

            thread::sleep(::std::time::Duration::from_millis(100));
        }
    }
}
