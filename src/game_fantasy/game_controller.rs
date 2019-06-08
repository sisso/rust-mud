mod view_login;
mod view_mainloop;

use super::game::*;
use crate::server;
use crate::server::ConnectionId;
use std::collections::{HashMap};

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
    // TODO: hashmap?
    connections: Vec<ConnectionState>,
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
}

// TODO: move login and input handling to utility
impl GameController {
    pub fn new(game: Game) -> Self {
        GameController {
            game,
            connections: vec![]
        }
    }

    pub fn connection_id_from_player_id(&self, player_id: &PlayerId) -> ConnectionId {
        for i in &self.connections {
            match i {
                ConnectionState::Logged { player_id: i_player_id, connection_id, .. } if i_player_id == player_id => {
                    return connection_id.clone()
                },
                _ => {}
            }
        }

        panic!("could not found connection for {}", player_id);
    }

    pub fn players_per_room(&self) -> HashMap<u32, Vec<PlayerId>> {
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

    pub fn player_id_from_connection_id(&self, connection_id: &ConnectionId) -> PlayerId {
        for i in &self.connections {
            match i {
                ConnectionState::Logged { player_id, connection_id: i_connection_id, .. } if i_connection_id == connection_id => {
                    return player_id.clone()
                },
                _ => {}
            }
        }

        panic!("could not found player for {}", connection_id);

    }

    pub fn handle(&mut self, connects: Vec<ConnectionId>, disconnects: Vec<ConnectionId>, inputs: Vec<(ConnectionId, String)>) -> Vec<server::Output> {
        let mut outputs = vec![];

        // handle new players
        for connection in connects {
            println!("gamecontroller - {} receive new player", connection.id);
            self.connections.push(ConnectionState::NotLogged {
                connection_id: connection,
            });

            let out = view_login::handle_welcome();
//            self.append_output(&mut outputs, HandleOutput::private(player_id, out));
            outputs.push(server::Output {
                dest_connections_id: vec![connection],
                output: out,
            });
        }

        // handle disconnected players
        for connection in disconnects {
            let index = self.connections.iter().position(|i| *i.connection_id() == connection).unwrap();
            match self.connections.get(index).unwrap() {
                ConnectionState::Logged { player_id, .. } => {
                    println!("gamecontroller - {} removing player {}", connection.id, player_id);
                    self.game.player_disconnect(&player_id);
                },
                ConnectionState::NotLogged {..} => {
                    println!("gamecontroller - {} removing non logged player", connection.id);
                },
            }
            self.connections.remove(index);
        }

        // handle players inputs
        for (connection_id, input) in inputs {
            let index = self.connections.iter().position(|i| *i.connection_id() == connection_id).unwrap();

            match self.connections.get(index).unwrap() {
                ConnectionState::Logged { connection_id, player_id, login, } => {
                    println!("gamecontroller - {} handling input '{}'", connection_id, input);
                    let out = view_mainloop::handle(&mut self.game, &player_id, &login, input);
                    self.append_outputs(&mut outputs, out);
                },
                ConnectionState::NotLogged {..} => {
                    println!("gamecontroller - {} handling login '{}'", connection_id, input);

                    let out = match view_login::handle(input) {
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

                            // update local state
                            self.connections.remove(index);
                            let new_connection_state = ConnectionState::Logged {
                                connection_id: connection_id,
                                player_id: player.id,
                                login: login.clone(),
                            };
                            self.connections.push(new_connection_state);

                            // handle output
                            let look_output = view_mainloop::handle_look(&self.game, &login);
                            format!("{}{}", out, look_output)
                        },
                        (_, out) => out,
                    };

                    outputs.push(server::Output {
                        dest_connections_id: vec![connection_id],
                        output: out,
                    });
                },
            }
        }

        outputs
    }

    fn append_output(&self, output: &mut Vec<server::Output>, handle_output: Output) {
        match handle_output {
            Output::Private { player_id, msg } => {
                let connection_id = self.connection_id_from_player_id(&player_id);

                output.push(server::Output {
                    dest_connections_id: vec![connection_id],
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
                            .filter(|i_player_id| player_id.filter(|j| *j != **i_player_id).is_some())
                            .map(|i_player_id| self.connection_id_from_player_id(i_player_id))
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
