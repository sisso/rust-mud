use std::thread;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::mpsc;

use crate::player_connection::*;
use crate::view_login;
use crate::game::*;
use crate::view_mainloop;

pub struct Server {
    connections: Vec<PlayerConnection>,
    nextId: u32,
    game: Game
}

impl Server {
    pub fn new(game: Game) -> Self {
        Server {
            connections: Vec::new(),
            nextId: 0,
            game: game
        }
    }

    // https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
    pub fn run(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        println!("Server listening on port 3333");

        let (sender, receiver) = mpsc::channel::<(u32, String, String)>();
        loop {
            if let Ok((mut stream, addr)) = listener.accept() {
                let id= self.nextId;
                self.nextId += 1;

                let sender = sender.clone();

                println!("New connection ({}): {}", id, addr);

                thread::spawn(move || {
                    // connection succeeded
                    let connection = Connection { stream: stream };
                    let player = view_login::handle_login(id, connection)
                        .expect("failed to handle connection login");
                    println!("Login complete for {}, user is '{}'", id, player.login);
//                        players.push(player);

                    sender.send((id, "player-connect".to_string(), player.login.clone()));

                    let _ = view_mainloop::handle(player);
                    sender.send((id, "player-disconnect".to_string(), "".to_string()));
                });
            }

            if let Ok((id, command, argument)) = receiver.try_recv() {
                match command.as_ref() {
                    "player-connect" => {
                        self.game.player_connect(id, argument);
                    }

                    "player-disconnect" => {
                        self.game.player_disconnect(id);
                    }

                    _ => {
                        println!("{} invalid command {}/{}", id, command, argument);
                    }
                }
            }

            thread::sleep(::std::time::Duration::from_millis(100));
        }

    }
}
