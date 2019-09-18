use std::collections::{HashMap, HashSet};

use crate::server;
use crate::server::ConnectionId;

use super::domain::*;
use super::container::Container;
use super::mob;
use super::item;
use super::player::PlayerId;
use super::room::RoomId;
use super::spawn;
use super::view_login;
use super::view_main;

use crate::utils::*;
use crate::utils::save::Save;

pub struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

pub struct Runner {
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
        room_id: RoomId,
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

    pub fn room(player_id: PlayerId, room_id: RoomId, msg: String) -> Self {
        Output::Room {
            player_id: Some(player_id),
            room_id,
            msg
        }
    }

    pub fn room_all(room_id: RoomId, msg: String) -> Self {
        Output::Room {
            player_id: None,
            room_id,
            msg
        }
    }
}

pub trait Outputs {
    fn room_all(&mut self, room_id: RoomId, msg: String);
    fn room(&mut self, player_id: PlayerId, room_id: RoomId, msg: String);
    fn private(&mut self, player_id: PlayerId, msg: String);
}

struct OutputsImpl {
    list: Vec<Output>
}

impl OutputsImpl {
    fn new() -> Self {
        OutputsImpl {
            list: vec![]
        }
    }
}

impl Outputs for OutputsImpl {
    fn room_all(&mut self, room_id: RoomId, msg: String) {
        self.list.push(Output::room_all(room_id, msg));
    }

    fn room(&mut self, player_id: PlayerId, room_id: RoomId, msg: String) {
        self.list.push(Output::room(player_id, room_id, msg));
    }

    fn private(&mut self, player_id: PlayerId, msg: String) {
        self.list.push(Output::private(player_id, msg));
    }
}

pub struct Ctx<'a> {
    pub time: &'a GameTime,
    pub container: &'a mut Container,
    pub outputs: &'a mut dyn Outputs
}

pub struct RunnerParams {
    pub connects: Vec<ConnectionId>,
    pub disconnects: Vec<ConnectionId>,
    pub inputs: Vec<(ConnectionId, String)>
}

impl Runner {
    pub fn new(container: Container) -> Self {
        Runner {
            container,
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
    pub fn handle(&mut self, time: GameTime, params: RunnerParams) -> Vec<server::Output> {
        let mut server_outputs: Vec<server::Output> = vec![];
        let mut outputs = OutputsImpl::new();
        let mut connections_with_input: HashSet<ConnectionId> = HashSet::new();

        // handle new players
        for connection_id in params.connects {
            info!("gamecontroller - {} receive new player", connection_id.id);
            self.connections.insert(connection_id.clone(), ConnectionState {
                connection_id,
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
            let state = self.get_state(connection);

            if let Some(player_id) = state.player_id {
                info!("gamecontroller - {} removing player {}", connection.id, player_id);
                self.container.players.player_disconnect(player_id);
            } else {
                info!("gamecontroller - {} removing non logged player", connection.id);
            }

            self.connections.remove(&connection);
        }

        // handle players inputs
        for (connection_id, input) in params.inputs {
            connections_with_input.insert(connection_id.clone());

            let state = self.get_state(connection_id);

            if let Some(player_id) = state.player_id {
                debug!("gamecontroller - {} handling input '{}'", connection_id, input);
                view_main::handle(&time, &mut self.container, &mut outputs, player_id, input);
            } else {
                debug!("gamecontroller - {} handling login '{}'", connection_id, input);
                let result = view_login::handle(&mut self.container, input);

                server_outputs.push(server::Output {
                    dest_connections_id: vec![connection_id],
                    output: result.msg
                });

                if let Some(player_id) = result.player_id {
                    debug!("gamecontroller - {} login complete for {}", connection_id, player_id);

                    self.set_state(ConnectionState {
                        connection_id: connection_id,
                        player_id: Some(player_id)
                    })
                }
            }
        }

        // run game tick
        {
            let mut ctx = Ctx {
                time: &time,
                container: &mut self.container,
                outputs: &mut outputs,
            };

            spawn::run(&mut ctx);
            mob::run_tick(&mut ctx);
            item::run_tick(&mut ctx);
        }

        self.append_outputs(&mut server_outputs, outputs);
        self.normalize_output(&mut server_outputs, &connections_with_input);

        server_outputs
    }

    pub fn save(&self, save: &mut dyn Save) {
        self.container.save(save);
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
                let connection_id = self.connection_id_from_player_id(player_id);

                output.push(server::Output {
                    dest_connections_id: vec![connection_id.clone()],
                    output: msg
                })

            },

            Output::Room { player_id, room_id, msg } => {
                debug!("game_controller - {:?}/{:?}: {}", player_id, room_id, msg);

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
                            .map(|i_player_id| self.connection_id_from_player_id(*i_player_id).clone())
                            .collect();

                    debug!("game_controller - players at room {:?}, selected connections: {:?}", players, connections_id);

                    output.push(server::Output {
                        dest_connections_id: connections_id,
                        output: msg
                    });
                } else {
                    debug!("game_controller - no players at room");
                }
            },

        }
    }

    fn append_outputs(&self, output: &mut Vec<server::Output>, handle_output: OutputsImpl) {
        for i in handle_output.list {
            self.append_output(output, i);
        }
    }

    fn connection_id_from_player_id(&self, player_id: PlayerId) -> &ConnectionId {
        self.connection_id_by_player_id
            .get(&player_id)
            .expect(format!("could not found connection for {}", player_id).as_str())
    }

    fn get_state(&self, connection_id: ConnectionId) -> &ConnectionState {
        self.connections
            .get(&connection_id)
            .expect(format!("could not found connection for {}", connection_id).as_str())
    }

    fn set_state(&mut self, state: ConnectionState) {
        if let Some(player_id) = state.player_id {
            self.connection_id_by_player_id.insert(player_id.clone(), state.connection_id.clone());
        }
        self.connections.insert(state.connection_id.clone(), state);

    }

    fn players_per_room(&self) -> HashMap<RoomId, Vec<PlayerId>> {
        let room_player: Vec<(RoomId, PlayerId)> =
            self.container.players.list_players()
                .into_iter()
                .map(|player_id| {
                    let player = self.container.players.get_player_by_id(player_id);
                    let avatar = self.container.mobs.get(&player.avatar_id);
                    (avatar.room_id, player_id)
                })
                .collect();

        // group_by
        let mut result: HashMap<RoomId, Vec<PlayerId>> = HashMap::new();
        for (room_id, player_id) in room_player {
            result.entry(room_id).or_insert(vec![]).push(player_id);
        }
        result
    }

    fn player_id_from_connection_id(&self, connection_id: &ConnectionId) -> Option<PlayerId> {
        let state = self.connections.get(connection_id).expect(format!("could not found state for connection {}", connection_id).as_str());
        state.player_id
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
