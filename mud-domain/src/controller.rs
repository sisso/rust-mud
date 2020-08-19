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

#[derive(Debug, Clone, Copy)]
pub enum ConnectionView {
    Login,
    Game,
    Admin,
}

#[derive(Debug)]
struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
    pub view: ConnectionView,
}

pub struct ViewHandleCtx<'a> {
    pub container: &'a mut Container,
    pub mob_id: MobId,
    pub player_id: PlayerId,
}

#[derive(Debug, Clone)]
pub enum ConnectionViewAction {
    None,
    Login(PlayerId),
    SwitchView(ConnectionView),
    Logout,
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
                view: ConnectionView::Login,
            },
        );

        let msg = view_login::handle_welcome();

        self.server_outputs.push((connection_id, msg));
    }

    pub fn disconnect(&mut self, container: &mut Container, connection_id: ConnectionId) {
        let state = match self.connections.get(&connection_id) {
            Some(state) => state,
            None => {
                warn!("disconnected connection {:?} not found", connection_id);
                return;
            }
        };

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

        let state = self
            .connections
            .get(&connection_id)
            .expect(format!("could not found connection {:?}", connection_id).as_str());

        let view_action: ConnectionViewAction = match state.view {
            ConnectionView::Login => {
                debug!("{:?} login input '{}'", connection_id, input);

                match view_login::handle(input) {
                    LoginResult::Msg { msg } => {
                        self.server_outputs.push((connection_id, msg));
                        ConnectionViewAction::None
                    }

                    LoginResult::Login { login } => {
                        self.server_outputs
                            .push((connection_id, view_login::on_login_success(login.as_str())));
                        // TODO: add login fail
                        let player_id =
                            avatars::on_player_login(container, login.as_str()).unwrap();

                        ConnectionViewAction::Login(player_id)
                    }
                }
            }

            ConnectionView::Game if state.player_id.is_none() => {
                warn!(
                    "{:?} is in Game view without a player id, changing view back to login",
                    connection_id
                );

                ConnectionViewAction::SwitchView(ConnectionView::Login)
            }

            ConnectionView::Game => {
                debug!("{:?} input '{}'", connection_id, input);

                let player_id = state.player_id.unwrap();

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
                    Err(ref err) if !err.is_failure() => {
                        warn!(
                            "{:?} exception handling input {:?}: {:?}",
                            connection_id, input, err
                        );

                        ConnectionViewAction::None
                    }
                    Err(_) => ConnectionViewAction::None,
                    Ok(action) => action,
                }
            }

            ConnectionView::Admin => ConnectionViewAction::None,
        };

        self.apply_action(container, connection_id, view_action);
    }

    fn apply_action(
        &mut self,
        _container: &mut Container,
        connection_id: ConnectionId,
        view_action: ConnectionViewAction,
    ) {
        match &view_action {
            ConnectionViewAction::None => {}
            _ => info!("{:?} executing {:?}", connection_id, view_action),
        }

        match view_action {
            ConnectionViewAction::None => {}

            ConnectionViewAction::SwitchView(view) => {
                let state = self.connections.get_mut(&connection_id).unwrap();
                state.view = view;
            }

            ConnectionViewAction::Login(player_id) => {
                debug!("{:?} login in {:?}", connection_id, player_id);

                let state = self.connections.get_mut(&connection_id).unwrap();
                state.view = ConnectionView::Game;
                state.player_id = Some(player_id);

                self.connection_id_by_player_id
                    .insert(player_id, connection_id);
            }

            ConnectionViewAction::Logout => {
                let state = self.connections.get_mut(&connection_id).unwrap();

                let old_player_id = state.player_id;
                debug!("{:?} log out {:?}", connection_id, old_player_id);

                state.view = ConnectionView::Login;
                state.player_id = None;

                if let Some(player_id) = old_player_id {
                    self.connection_id_by_player_id.remove(&player_id);
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

        // A $ symbol can be added to indicate user input
        // for connection_id in append_cursor_ids {
        //     self.server_outputs.push((connection_id, "$ ".to_string()));
        // }
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

    pub fn player_id_from_connection_id(&self, connection_id: ConnectionId) -> Option<PlayerId> {
        self.connections
            .get(&connection_id)
            .and_then(|i| i.player_id)
    }
}
