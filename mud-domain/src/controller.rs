use commons::*;
use logs::*;
use std::collections::{HashMap, HashSet};
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::{Outputs, avatars};
use crate::game::container::Container;
use crate::controller::view_login::LoginResult;

pub mod view_login;
pub mod view_main;

#[derive(Debug)]
struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

#[derive(Debug)]
enum Output {
    Private {
        mob_id: MobId,
        msg: String,
    },

    Broadcast {
        /// usually the mob that originate the message
        exclude: Option<MobId>,
        /// RoomId or ZoneId, all children mobs will receive the message
        location_id: LocationId,
        /// recursive search for mobs to send message
        recursive: bool,
        msg: String,
    },
}

#[derive(Debug)]
pub struct OutputsBuffer {
    list: Vec<Output>,
}

impl OutputsBuffer {
    pub fn new() -> Self {
        OutputsBuffer { list: vec![] }
    }

    fn take(&mut self) -> Vec<Output> {
        std::mem::replace(&mut self.list, vec![])
    }
}

impl Outputs for OutputsBuffer {
    fn broadcast_all(&mut self, exclude: Option<ObjId>, location_id: ObjId, msg: String) {
        self.list.push(Output::Broadcast {
            exclude,
            location_id,
            msg,
            recursive: true,
        })
    }

    fn broadcast(&mut self, exclude: Option<ObjId>, location_id: ObjId, msg: String) {
        self.list.push(Output::Broadcast {
            exclude,
            location_id,
            msg,
            recursive: false,
        })
    }

    fn private(&mut self, mob_id: MobId, msg: String) {
        self.list.push(Output::Private{
            mob_id,
            msg
        })
    }
}

pub struct Controller {
    connections: HashMap<ConnectionId, ConnectionState>,
    connection_id_by_player_id: HashMap<PlayerId, ConnectionId>,
    server_outputs: Vec<(ConnectionId, String)>,
    outputs: OutputsBuffer,
    connections_with_input: HashSet<ConnectionId>,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            connections: Default::default(),
            connection_id_by_player_id: Default::default(),
            server_outputs: Default::default(),
            outputs: OutputsBuffer::new(),
            connections_with_input: Default::default(),
        }
    }

    pub fn add_connection(&mut self, container: &mut Container, connection_id: ConnectionId) {
        info!("{:?} receive connection", connection_id);
        self.connections.insert(
            connection_id.clone(),
            ConnectionState {
                connection_id,
                player_id: None,
            },
        );

        let msg = view_login::handle_welcome();

        self.server_outputs.push((connection_id, msg));
    }

    pub fn disconnect(&mut self, container: &mut Container, connection_id: ConnectionId) {
        let state = self.get_state(connection_id);

        if let Some(player_id) = state.player_id {
            info!("{:?} disconnecting player {:?}", connection_id, player_id);
            avatars::on_player_disconnect(container, &mut self.outputs, player_id);
        } else {
            info!("{:?} disconnecting", connection_id);
        }

        self.connections.remove(&connection_id);
    }

    pub fn handle_input(&mut self, container: &mut Container, connection_id: ConnectionId, input: &str) {
        self.connections_with_input.insert(connection_id);

        let state = self.get_state(connection_id);

        if let Some(player_id) = state.player_id {
            debug!("{:?} handling input '{}'", connection_id, input);
            let mob_id = container.players.get(player_id).mob_id;
            let _ = view_main::handle(container, &mut self.outputs, mob_id, input);
        } else {
            debug!("{:?} handling login '{}'", connection_id, input);
            match view_login::handle(input) {
                LoginResult::Msg { msg } => {
                    self.server_outputs.push((connection_id, msg));
                }
                LoginResult::Login { login } => {
                    self.server_outputs
                        .push((connection_id, view_login::on_login_success(login.as_str())));
                    // TODO: add login fail
                    let player_id = avatars::on_player_login(
                        container,
                        &mut self.outputs,
                        login.as_str(),
                    ).unwrap();

                    debug!("{:?} login complete for {:?}", connection_id, player_id);
                    self.set_state(ConnectionState {
                        connection_id,
                        player_id: Some(player_id),
                    })
                }
            }
        }
    }


   pub fn get_outputs(&mut self) -> &mut dyn Outputs {
       &mut self.outputs
   }

   pub fn flush_outputs(&mut self, container: &Container) -> Vec<(ConnectionId, String)> {
        self.convert_to_connections_output(container);
        self.normalize_connection_outputs(container);

        // clear temporary
        self.connections_with_input.clear();

        // return outputs
        std::mem::replace(&mut self.server_outputs, vec![])
    }

    //    pub fn save(&self, save: &mut dyn Save) {
    //        container.save(save);
    //    }

    /// For each player that will receive output, append new line with cursor.
    ///
    /// If player send no input, append a new line before any output
    fn normalize_connection_outputs(&mut self, container: &Container) {
        let mut append_cursor_ids: Vec<ConnectionId> = vec![];
        let mut new_lines_ids: Vec<ConnectionId> = vec![];

        for (connection_id, _) in self.server_outputs.iter() {
            let player_id = self.player_id_from_connection_id(connection_id);

            match player_id {
                Some(_) if !append_cursor_ids.contains(connection_id) => {
                    append_cursor_ids.push(connection_id.clone());

                    // if player do not have newline because sent a input, append new line in start
                    if !self.connections_with_input.contains(connection_id)
                        && !new_lines_ids.contains(&connection_id)
                    {
                        new_lines_ids.push(connection_id.clone());
                    }
                }
                _ => {}
            }
        }

        for connection_id in new_lines_ids {
            self.server_outputs
                .insert(0, (connection_id, "\n".to_string()));
        }

        for connection_id in append_cursor_ids {
            self.server_outputs.push((connection_id, "\n".to_string()));
        }
    }

    /// Convert game output into connection output.
    ///
    /// Redirect private msg to specific player connections and room messages to players
    /// in room connections.
    ///
    /// game output will be empty after this process
    fn convert_to_connections_output(&mut self, container: &Container) {
        let outputs = self.outputs.take();

        for game_output in outputs {
            match game_output {
                Output::Private { mob_id, msg } => {
                    let connection_id = container.players.find_from_mob(mob_id)
                        .and_then(|player_id| self.zip_connection_id_from_player_id(player_id));

                    if let Some((player_id, connection_id)) = connection_id {
                        debug!("{:?} - {}", player_id, msg);
                        self.server_outputs.push((connection_id, format!("{}\n", msg)));
                    }
                }

                Output::Broadcast {
                    exclude,
                    location_id,
                    recursive,
                    msg
                } => {
                    let exclude_player = exclude.and_then(|mob_id| container.players.find_from_mob(mob_id));

                    let players: Vec<PlayerId> =
                        if recursive {
                            avatars::find_deep_players_in(&container, location_id)
                        } else {
                            avatars::find_players_in(&container, location_id)
                        };

                    let connections: Vec<(PlayerId, ConnectionId)> = players.into_iter()
                        .filter(|&player_id| Some(player_id) != exclude_player)
                        .flat_map(|player_id| self.zip_connection_id_from_player_id(player_id))
                        .collect();

                    for (player_id, connection_id) in connections {
                        debug!("{:?} - {}", player_id, msg);
                        self.server_outputs.push((connection_id, format!("{}\n", msg)))
                    }
                }
            }
        }
    }

    fn zip_connection_id_from_player_id(&self, player_id: PlayerId) -> Option<(PlayerId, ConnectionId)> {
        self.connection_id_from_player_id(player_id).map(|connection_id| {
            (player_id, connection_id)
        })
    }

    fn connection_id_from_player_id(&self, player_id: PlayerId) -> Option<ConnectionId> {
        self.connection_id_by_player_id.get(&player_id).cloned()
    }

    fn get_state(&self, connection_id: ConnectionId) -> &ConnectionState {
        self.connections
            .get(&connection_id)
            .expect(format!("could not found connection for {:?}", connection_id).as_str())
    }

    fn set_state(&mut self, state: ConnectionState) {
        if let Some(player_id) = state.player_id {
            self.connection_id_by_player_id
                .insert(player_id.clone(), state.connection_id.clone());
        }
        self.connections.insert(state.connection_id.clone(), state);
    }

    fn player_id_from_connection_id(&self, connection_id: &ConnectionId) -> Option<PlayerId> {
        self.connections
            .get(connection_id)
            .and_then(|i| i.player_id)
    }
}
