mod view_login;
mod view_mainloop;

use super::game::*;
use crate::server;
use crate::server::ConnectionId;
use std::collections::{HashMap, HashSet};

enum ConnectionState {
    NotLogged {
        connection_id: ConnectionId,
    },
    Logged {
        connection_id: ConnectionId,
        player_id: PlayerId,
        login: String,
    }
}

impl ConnectionState {
    fn connection_id(&self) -> &ConnectionId {
        match self {
            ConnectionState::NotLogged { connection_id, .. } => connection_id,
            ConnectionState::Logged { connection_id, .. } => connection_id
        }
    }
}

pub struct GameController {
    game: Game,
    connections: HashMap<ConnectionId, ConnectionState>,
    connection_id_by_player_id: HashMap<PlayerId, ConnectionId>,
}

pub enum Output {
    Private {
        player_id: PlayerId,
        msg: String
    },

    Room {
        /// player that originate the message, he is the only one will not receive the message
        player_id: Option<PlayerId>,
        room_id: u32,
        msg: String
    }
}

impl Output {
    pub fn private(player_id: PlayerId, msg: String) -> Self {
        Output::Private {
            player_id,
            msg
        }
    }

    pub fn room(player_id: PlayerId, room_id: u32, msg: String) -> Self {
        Output::Room {
            player_id: Some(player_id),
            room_id,
            msg
        }
    }

    pub fn player_id(&self) -> Option<PlayerId> {
        match self {
            Output::Private { player_id, .. } => Some(player_id.clone()),
            Output::Room { player_id, .. } => player_id.clone(),
        }
    }
}

// TODO: move login and input handling to utility
impl GameController {
    pub fn new(game: Game) -> Self {
        GameController {
            game,
            connections: HashMap::new(),
            connection_id_by_player_id: HashMap::new()
        }
    }

    //
    // 1. new connections
    // 2. disconnects
    // 3. inputs
    // 4. actions
    // 5. outputs
    //
    pub fn handle(&mut self, connects: Vec<ConnectionId>, disconnects: Vec<ConnectionId>, inputs: Vec<(ConnectionId, String)>) -> Vec<server::Output> {
        let mut server_outputs: Vec<server::Output> = vec![];
        let mut outputs: Vec<Output> = vec![];

        // handle new players
        for connection in connects {
            println!("gamecontroller - {} receive new player", connection.id);
            self.connections.insert(connection.clone(), ConnectionState::NotLogged {
                connection_id: connection,
            });

            let out = view_login::handle_welcome();
            server_outputs.push(server::Output {
                dest_connections_id: vec![connection],
                output: out,
            });
        }

        // handle disconnected players
        for connection in disconnects {
            let state = self.get_state(&connection);
            match state {
                ConnectionState::Logged { player_id, .. } => {
                    println!("gamecontroller - {} removing player {}", connection.id, player_id);
                    let player_id = *player_id;
                    self.game.player_disconnect(&player_id);
                },
                ConnectionState::NotLogged {..} => {
                    println!("gamecontroller - {} removing non logged player", connection.id);
                },
            }
            self.connections.remove(&connection);
        }

        // handle players inputs
        for (connection_id, input) in inputs {
            let state = self.get_state(&connection_id);
            match state {
                ConnectionState::Logged { connection_id, player_id, .. } => {
                    println!("gamecontroller - {} handling input '{}'", connection_id, input);
                    let player_id = *player_id;
                    let mut out = view_mainloop::handle(&mut self.game, &player_id, input);
                    outputs.append(&mut out);
                },

                ConnectionState::NotLogged {..} => {
                    println!("gamecontroller - {} handling login '{}'", connection_id, input);

                    match view_login::handle(input) {
                        (Some(login), out) => {
                            // add player avatar
                            let mob_id = self.game.next_mob_id();
                            let mob = Mob {
                                id: mob_id,
                                label: login.clone(),
                                room_id: 0,
                                is_avatar: true
                            };
                            self.game.add_mob(mob);

                            // add player to game
                            let player = self.game.player_connect(login.clone(), mob_id);
                            let player_id = player.id;

                            // update local state
                            self.connections.remove(&connection_id);
                            let new_connection_state = ConnectionState::Logged {
                                connection_id: connection_id,
                                player_id: player_id,
                                login: login.clone(),
                            };
                            self.connections.insert(connection_id, new_connection_state);
                            self.connection_id_by_player_id.insert(player_id, connection_id);

                            // handle output
                            let look_output = view_mainloop::handle_look(&self.game, &player_id);
                            outputs.push(Output::private(player_id, format!("{}{}", out, look_output)));
                        },
                        (_, out) => {
                            server_outputs.push(server::Output {
                                dest_connections_id: vec![connection_id],
                                output: out,
                            });
                        },
                    };
                },
            }
        }

        self.append_outputs(&mut server_outputs, outputs);
        self.append_cursors(&mut server_outputs);
        server_outputs
    }

    /// For each player that will receive output, append new line with cursor
    /// TODO: how to handle the case that user receive a output without giving a input (with enter new line)?
    ///       should we added a line before any output?
    fn append_cursors(&self, outputs: &mut Vec<server::Output>) {
        let mut output_to_add : Vec<ConnectionId> = vec![];

        for i in outputs.iter() {
            for j in &i.dest_connections_id {
                let player_id = self.player_id_from_connection_id(j);

                match player_id {
                    Some(player_id) if !output_to_add.contains(j) => {
                        output_to_add.push(j.clone());
                    },
                    _ => {},
                }
            }
        }

        outputs.push(server::Output {
            dest_connections_id: output_to_add,
            output: "\n$ ".to_string()
        });
    }

//    fn append_cursors(&self, outputs: &mut Vec<Output>) {
//        let mut already_added_set: HashSet<PlayerId> = HashSet::new();
//        let mut output_to_add : Vec<Output> = vec![];
//        outputs.iter_mut().for_each(|i| {
//            match i.player_id() {
//                Some(player_id) if !already_added_set.contains(&player_id) => {
//                    already_added_set.insert(player_id.clone());
//                    output_to_add.push(Output::private(player_id, "\n$ ".to_string()));
//                },
//                _ => {} // ignore
//            }
//        });
//
//        outputs.append(&mut output_to_add);
//    }

    fn append_output(&self, output: &mut Vec<server::Output>, handle_output: Output) {
        match handle_output {
            Output::Private { player_id, msg } => {
                let connection_id = self.connection_id_from_player_id(&player_id);

                output.push(server::Output {
                    dest_connections_id: vec![connection_id.clone()],
                    output: msg
                })

            },

            Output::Room { player_id, room_id, msg } => {
                println!("game_controller - {:?}/{}: {}", player_id, room_id, msg);

                let players_per_room = self.players_per_room();

                if let Some(players) = players_per_room.get(&room_id) {
                    let connections_id: Vec<ConnectionId> =
                        players
                            .iter()
                            // exclude player that emit the message from receivers
                            .filter(|i_player_id| player_id.filter(|j| *j != **i_player_id).is_some())
                            .map(|i_player_id| self.connection_id_from_player_id(i_player_id).clone())
                            .collect();

                    println!("game_controller - players at room {:?}, selected connections: {:?}", players, connections_id);

                    output.push(server::Output {
                        dest_connections_id: connections_id,
                        output: msg
                    });
                } else {
                    println!("game_controller - no players at room");
                }
            },

        }
    }

    fn append_outputs(&self, output: &mut Vec<server::Output>, handle_output: Vec<Output>) {
        for i in handle_output {
            self.append_output(output, i);
        }
    }

    fn connection_id_from_player_id(&self, player_id: &PlayerId) -> &ConnectionId {
        self.connection_id_by_player_id
            .get(player_id)
            .expect(format!("could not found connection for {}", player_id).as_str())
    }

    fn get_state(&self, connection_id: &ConnectionId) -> &ConnectionState {
        self.connections
            .get(connection_id)
            .expect(format!("could not found connection for {}", connection_id).as_str())
    }

    fn players_per_room(&self) -> HashMap<u32, Vec<PlayerId>> {
        let room_player: Vec<(u32, PlayerId)> =
            self.game.list_players()
                .into_iter()
                .map(|player_id| {
                    let player = self.game.get_player_by_id(player_id);
                    let avatar = self.game.get_mob(player.avatar_id);
                    (avatar.room_id, *player_id)
                })
                .collect();

        // group_by
        let mut result: HashMap<u32, Vec<PlayerId>> = HashMap::new();
        for (room_id, player_id) in room_player {
            result.entry(room_id).or_insert(vec![]).push(player_id);
        }
        result
    }

    fn player_id_from_connection_id(&self, connection_id: &ConnectionId) -> Option<&PlayerId> {
        let state = self.connections.get(connection_id).expect(format!("could not found state for connection {}", connection_id).as_str());
        match state {
            ConnectionState::Logged { player_id, .. } => Some(player_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r_mob(game: &mut Game, label: String) -> &Mob {
        let mob_id = game.next_mob_id();

        let mob = Mob {
            id: mob_id,
            label: label,
            room_id: 0,
            is_avatar: true
        };

        game.add_mob(mob)
    }

    #[test]
    fn test_players_per_room() {
        let mut game = Game::new();

        game.add_room(Room {
            id: 0,
            label: "room1".to_string(),
            desc: "".to_string(),
            exits: vec![],
        });

        let mob_player_0_id = r_mob(&mut game,"player0".to_string()).id;
        let mob_player_1_id = r_mob(&mut game,"player1".to_string()).id;

        game.player_connect("player0".to_string(), mob_player_0_id);
        game.player_connect( "player1".to_string(), mob_player_1_id);

        let gc = GameController::new(game);

        let map = gc.players_per_room();
        let result = map.get(&0);
        println!("{:?}", result);
        assert_eq!(result, Some(&vec![PlayerId(0), PlayerId(1)]));
    }
}
