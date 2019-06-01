use std::thread;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::mpsc;

use crate::player_connection::Connection;
use crate::view_login;
use crate::game::*;
use crate::view_mainloop;

pub struct Server {
    connections: Vec<Connection>,
    game: Game
}

impl Server {
    pub fn new(game: Game) -> Self {
        Server {
            connections: Vec::new(),
            game: game
        }
    }

    // https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
    pub fn run(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        println!("Server listening on port 3333");

        let mut next_id = 0;

        let mut broken_connections: Vec<u32> = vec![];
        let mut inputs : Vec<(u32, String)> = vec![];

        loop {
            // accept new connections
            if let Ok((mut stream, addr)) = listener.accept() {
                let id= next_id;
                next_id += 1;

                println!("new connection ({}) {}, total connections {}", addr, id, self.connections.len());
                stream.set_nonblocking(true)
                    .expect(format!("failed to set non_blocking stream for {}", id).as_str());

                // connection succeeded
                let connection = Connection {
                    id: id,
                    stream: stream,
                    login: None
                };

                self.connections.push(connection);

//                let player = view_login::handle_login(id, connection)
//                    .expect("failed to handle connection login");
//                println!("Login complete for {}, user is '{}'", id, player.login);
////                        players.push(player);
//
//                sender.send((id, "player-connect".to_string(), player.login.clone()));
//
//                let _ = view_mainloop::handle(player);
//                sender.send((id, "player-disconnect".to_string(), "".to_string()));
            }

            // handle inputs
            for connection in &mut self.connections {
                match Connection::read_line(&mut connection.stream) {
                    Ok(line) => {
                        inputs.push((connection.id, line));
                    },
                    Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => (),
                    Err(e) => {
                        println!("{} failed: {}", connection.id, e);
                        broken_connections.push(connection.id)
                    }
                }
            }

            // remove broken connections
            for id in &broken_connections {
                let index = self.connections.iter().position(|i| i.id == *id).unwrap();
                self.connections.remove(index);

                println!("{} removed, total connections {}", *id, self.connections.len());
            }
            broken_connections.clear();

            // update game

            // handle outputs
            for connection in &mut self.connections {
//                let (id, stream, maybe_login) = match connection {
//                    Connection::NewConnection { id: id, stream: stream} => (id, stream, None)
//                    Connection::PlayerConnection { id: id, stream: stream, login: login } => (id, stream, Some(login))
//
//                }

                for input in &mut inputs {
                    if input.0 != connection.id  {
                        Connection::writeln(&mut connection.stream, input.1.as_str());
                    }
                }
            }

            inputs.clear();

//            if let Ok((id, command, argument)) = receiver.try_recv() {
//                match command.as_ref() {
//                    "player-connect" => {
//                        self.game.player_connect(id, argument);
//                    }
//
//                    "player-disconnect" => {
//                        self.game.player_disconnect(id);
//                    }
//
//                    _ => {
//                        println!("{} invalid command {}/{}", id, command, argument);
//                    }
//                }
//            }

            // TODO: create time ticket
            thread::sleep(::std::time::Duration::from_millis(100));
        }

    }
}
