use crate::server;
use crate::server::ConnectionId;

use super::domain::*;
use super::spawn;
use super::view_main;
use super::view_login;

use std::collections::{HashMap, HashSet};

pub struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

pub struct GameController {
    container: Container,
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

    pub fn room_all(room_id: u32, msg: String) -> Self {
        Output::Room {
            player_id: None,
            room_id,
            msg
        }
    }
}

pub struct GameControllerContext {
    pub connects: Vec<ConnectionId>,
    pub disconnects: Vec<ConnectionId>,
    pub inputs: Vec<(ConnectionId, String)>
}

impl GameController {
    pub fn new(container: Container) -> Self {
        GameController {
            container: container,
            connections: HashMap::new(),
            connection_id_by_player_id: HashMap::new(),
        }
    }

    //
    // 1. new connections
    // 2. disconnects
    // 3. inputs
    // 4. actions
    // 5. outputs
    //
    pub fn handle(&mut self, params: GameControllerContext) -> Vec<server::Output> {
        let mut server_outputs: Vec<server::Output> = vec![];
        let mut outputs: Vec<Output> = vec![];
        let mut connections_with_input: HashSet<ConnectionId> = HashSet::new();

        // TODO fix time
        self.container.update_tick(Seconds(100.0 / 1000.0));

        // handle new players
        for connection_id in params.connects {
            println!("gamecontroller - {} receive new player", connection_id.id);
            self.connections.insert(connection_id.clone(), ConnectionState {
                connection_id: connection_id,
                player_id: None
            });

            let msg = view_login::handle_welcome();
            server_outputs.push(server::Output {
                dest_connections_id: vec![connection_id.clone()],
                output: msg,
            });
        }

        // handle disconnected players
        for connection in params.disconnects {
            let state = self.get_state(&connection);

            if let Some(player_id) = state.player_id {
                println!("gamecontroller - {} removing player {}", connection.id, player_id);
                self.container.player_disconnect(&player_id);
            } else {
                println!("gamecontroller - {} removing non logged player", connection.id);
            }

            self.connections.remove(&connection);
        }

        // handle players inputs
        for (connection_id, input) in params.inputs {
            connections_with_input.insert(connection_id.clone());

            let state = self.get_state(&connection_id);

            if let Some(player_id) = state.player_id {
                println!("gamecontroller - {} handling input '{}'", connection_id, input);
                view_main::handle(&mut self.container, &mut outputs, &player_id, input);
            } else {
                println!("gamecontroller - {} handling login '{}'", connection_id, input);
                let result = view_login::handle(&mut self.container, input);

                server_outputs.push(server::Output {
                    dest_connections_id: vec![connection_id],
                    output: result.msg
                });

                if let Some(player_id) = result.player_id {
                    println!("gamecontroller - {} login complete for {}", connection_id, player_id);

                    self.set_state(ConnectionState {
                        connection_id: connection_id,
                        player_id: Some(player_id)
                    })
                }
            }
        }

        spawn::run(&mut self.container, &mut outputs);

        self.append_outputs(&mut server_outputs, outputs);
        self.normalize_output(&mut server_outputs, &connections_with_input);
        server_outputs
    }

    /// For each player that will receive output, append new line with cursor.
    ///
    /// If player send no input, append a new line before any output
    fn normalize_output(&self, outputs: &mut Vec<server::Output>, connections_with_input: &HashSet<ConnectionId>) {
        let mut append_cursor_ids: Vec<ConnectionId> = vec![];
        let mut new_lines_ids: Vec<ConnectionId> = vec![];

        for output in outputs.iter() {
            for connection in &output.dest_connections_id {
                let player_id = self.player_id_from_connection_id(connection);

                match player_id {
                    Some(_) if !append_cursor_ids.contains(connection) => {
                        append_cursor_ids.push(connection.clone());

                        // if player do not have newline because sent a input, append new line in start
                        if !connections_with_input.contains(connection) && !new_lines_ids.contains(&connection) {
                            new_lines_ids.push(connection.clone());
                        }
                    },
                    _ => {},
                }
            }
        }

        outputs.insert(0, server::Output {
            dest_connections_id: new_lines_ids,
            output: "\n".to_string()
        });

        outputs.push(server::Output {
            dest_connections_id: append_cursor_ids,
            output: "\n$ ".to_string()
        });
    }

    /// Convert controller output into server output. Redirect private msg to specific player
    /// connections and room messages to players in room connections.
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
                            .filter(|i_player_id| {
                                match player_id {
                                    // exclude player that emit the message from receivers
                                    Some(player_id) if **i_player_id == player_id => false,
                                    _ => true
                                }
                            })
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

    fn set_state(&mut self, state: ConnectionState) {
        if let Some(player_id) = state.player_id {
            self.connection_id_by_player_id.insert(player_id.clone(), state.connection_id.clone());
        }
        self.connections.insert(state.connection_id.clone(), state);

    }

    fn players_per_room(&self) -> HashMap<u32, Vec<PlayerId>> {
        let room_player: Vec<(u32, PlayerId)> =
            self.container.list_players()
                .into_iter()
                .map(|player_id| {
                    let player = self.container.get_player_by_id(player_id);
                    let avatar = self.container.get_mob(&player.avatar_id);
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
        state.player_id.as_ref()
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn sample_test() {
//        assert_eq!(true, true);
//    }
//}
