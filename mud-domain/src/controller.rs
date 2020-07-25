use crate::controller::view_login::LoginResult;
use crate::game::avatars;
use crate::game::container::Container;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::outputs::{Output, Outputs};
use commons::*;
use logs::*;
use std::collections::{HashMap, HashSet};

mod input_handle_hire;
mod input_handle_items;
mod input_handle_space;
mod input_handle_vendors;
mod view_login;
mod view_main;

#[derive(Debug)]
struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

pub struct ViewHandleCtx<'a> {
    pub container: &'a mut Container,
    pub mob_id: MobId,
    pub player_id: PlayerId,
}

/// Manage connectivity and messages to players through a socket.
pub struct Controller {
    connections: HashMap<ConnectionId, ConnectionState>,
    connection_id_by_player_id: HashMap<PlayerId, ConnectionId>,
    server_outputs: Vec<(ConnectionId, String)>,
    connections_with_input: HashSet<ConnectionId>,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            connections: Default::default(),
            connection_id_by_player_id: Default::default(),
            server_outputs: Default::default(),
            connections_with_input: Default::default(),
        }
    }

    pub fn add_connection(&mut self, _container: &mut Container, connection_id: ConnectionId) {
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
            avatars::on_player_disconnect(container, player_id);
        } else {
            info!("{:?} disconnecting", connection_id);
        }

        self.connections.remove(&connection_id);
    }

    // TODO: should not trigger changes in container, but just append inputs
    pub fn handle_input(
        &mut self,
        container: &mut Container,
        connection_id: ConnectionId,
        input: &str,
    ) {
        self.connections_with_input.insert(connection_id);

        let state = self.get_state(connection_id);

        if let Some(player_id) = state.player_id {
            debug!("{:?} input '{}'", connection_id, input);

            let mob_id = container
                .players
                .get(player_id)
                .expect("player not found")
                .mob_id;

            let ctx = ViewHandleCtx {
                container: container,
                mob_id,
                player_id,
            };

            match view_main::handle(ctx, input) {
                Err(ref err) if !err.is_failure() => warn!(
                    "{:?} exception handling input {:?}: {:?}",
                    connection_id, input, err
                ),
                _ => {}
            }
        } else {
            debug!("{:?} login input '{}'", connection_id, input);
            match view_login::handle(input) {
                LoginResult::Msg { msg } => {
                    self.server_outputs.push((connection_id, msg));
                }
                LoginResult::Login { login } => {
                    self.server_outputs
                        .push((connection_id, view_login::on_login_success(login.as_str())));
                    // TODO: add login fail
                    let player_id = avatars::on_player_login(container, login.as_str()).unwrap();

                    debug!("{:?} login complete for {:?}", connection_id, player_id);
                    self.set_state(ConnectionState {
                        connection_id,
                        player_id: Some(player_id),
                    })
                }
            }
        }
    }

    pub fn flush_outputs(&mut self, container: &mut Container) -> Vec<(ConnectionId, String)> {
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
    fn normalize_connection_outputs(&mut self, _container: &Container) {
        let mut append_cursor_ids: Vec<ConnectionId> = vec![];
        let mut new_lines_ids: Vec<ConnectionId> = vec![];

        for (connection_id, _) in self.server_outputs.iter().cloned() {
            let player_id = self.player_id_from_connection_id(connection_id);

            match player_id {
                Some(_) if !append_cursor_ids.contains(&connection_id) => {
                    append_cursor_ids.push(connection_id.clone());

                    // if player do not have newline because sent a input, append new line in start
                    if !self.connections_with_input.contains(&connection_id)
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
    fn convert_to_connections_output(&mut self, container: &mut Container) {
        let outputs = container.outputs.take();

        for game_output in outputs {
            match game_output {
                Output::Private { mob_id, msg } => {
                    let connection_id = container
                        .players
                        .find_from_mob(mob_id)
                        .and_then(|player_id| self.zip_connection_id_from_player_id(player_id));

                    if let Some((_player_id, connection_id)) = connection_id {
                        debug!("{:?} output {:?}", connection_id, msg);
                        self.server_outputs
                            .push((connection_id, format!("{}\n", msg)));
                    }
                }

                Output::Broadcast {
                    exclude,
                    location_id,
                    recursive,
                    msg,
                } => {
                    let exclude_player =
                        exclude.and_then(|mob_id| container.players.find_from_mob(mob_id));

                    let players: Vec<PlayerId> = if recursive {
                        avatars::find_deep_players_in(&container, location_id)
                    } else {
                        avatars::find_players_in(&container, location_id)
                    };

                    let connections: Vec<(PlayerId, ConnectionId)> = players
                        .into_iter()
                        .filter(|&player_id| Some(player_id) != exclude_player)
                        .flat_map(|player_id| self.zip_connection_id_from_player_id(player_id))
                        .collect();

                    for (_player_id, connection_id) in connections {
                        debug!("{:?} output {:?}", connection_id, msg);
                        self.server_outputs
                            .push((connection_id, format!("{}\n", msg)))
                    }
                }
            }
        }
    }

    fn zip_connection_id_from_player_id(
        &self,
        player_id: PlayerId,
    ) -> Option<(PlayerId, ConnectionId)> {
        self.connection_id_from_player_id(player_id)
            .map(|connection_id| (player_id, connection_id))
    }

    pub fn connection_id_from_player_id(&self, player_id: PlayerId) -> Option<ConnectionId> {
        self.connection_id_by_player_id.get(&player_id).cloned()
    }

    fn get_state(&self, connection_id: ConnectionId) -> &ConnectionState {
        self.connections
            .get(&connection_id)
            .expect(format!("could not found connection {:?}", connection_id).as_str())
    }

    fn set_state(&mut self, state: ConnectionState) {
        if let Some(player_id) = state.player_id {
            self.connection_id_by_player_id
                .insert(player_id.clone(), state.connection_id.clone());
        }
        self.connections.insert(state.connection_id.clone(), state);
    }

    pub fn player_id_from_connection_id(&self, connection_id: ConnectionId) -> Option<PlayerId> {
        self.connections
            .get(&connection_id)
            .and_then(|i| i.player_id)
    }
}
