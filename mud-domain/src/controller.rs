use crate::controller::view_login::LoginResult;
use crate::errors::AsResult;
use crate::errors::{Error, Result};
use crate::game::avatars;
use crate::game::combat::kill_mob;
use crate::game::container::Container;
use crate::game::inventory_service::compute_total_weight;
use crate::game::loader::dto::{ObjData, StaticId};
use crate::game::loader::Loader;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::outputs::{OMarker, Output, Outputs};
use commons::asciicolors;
use commons::*;
use logs::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

mod input_handle_command;
mod input_handle_hire;
mod input_handle_items;
mod input_handle_space;
mod input_handle_vendors;
mod view_admin;
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
pub struct ConnectionController {
    connections: HashMap<ConnectionId, ConnectionState>,
    connection_id_by_player_id: HashMap<PlayerId, ConnectionId>,
    server_outputs: Vec<(ConnectionId, String)>,
    connections_with_input: HashSet<ConnectionId>,
}

impl ConnectionController {
    pub fn new() -> Self {
        ConnectionController {
            connections: Default::default(),
            connection_id_by_player_id: Default::default(),
            server_outputs: Default::default(),
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
                view: ConnectionView::Login,
            },
        );

        if let Err(e) = self.apply_action(
            container,
            connection_id,
            ConnectionViewAction::SwitchView(ConnectionView::Login),
        ) {
            warn!(
                "{:?} error when change view to login: {:?}",
                connection_id, e
            );
        }
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

    // TODO: should not trigger changes in container, but just append events inputs?
    //       q1: and about admin?
    // TODO: normalize views interface? login/admin per connection and game per mob_id?
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

        let view_action: crate::errors::Result<ConnectionViewAction> = match state.view {
            ConnectionView::Login => {
                debug!("{:?} login input '{}'", connection_id, input);

                match view_login::handle(input) {
                    LoginResult::Msg { msg } => {
                        self.server_outputs.push((connection_id, msg));
                        Ok(ConnectionViewAction::None)
                    }

                    LoginResult::Login { login } => {
                        self.server_outputs
                            .push((connection_id, view_login::on_login_success(login.as_str())));
                        // TODO: add login fail
                        let player_id =
                            avatars::on_player_login(container, login.as_str()).unwrap();

                        Ok(ConnectionViewAction::Login(player_id))
                    }
                }
            }

            ConnectionView::Game if state.player_id.is_none() => {
                warn!(
                    "{:?} is in Game view without a player id, changing view back to login",
                    connection_id
                );

                Ok(ConnectionViewAction::SwitchView(ConnectionView::Login))
            }

            ConnectionView::Game => {
                debug!("{:?} input '{}'", connection_id, input);

                let player_id = state.player_id.expect("game view must have a player");

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

                view_main::handle(ctx, input)
            }

            ConnectionView::Admin => {
                let mut outputs = vec![];
                let result = view_admin::handle(container, &mut outputs, input);
                for msg in outputs {
                    self.server_outputs.push((connection_id, msg));
                    self.server_outputs.push((connection_id, "\n".to_string()));
                }
                result
            }
        };

        match view_action.and_then(|action| self.apply_action(container, connection_id, action)) {
            Err(ref err) if !err.is_failure() => {
                warn!(
                    "{:?} exception handling input {:?}: {:?}",
                    connection_id, input, err
                );
            }

            Err(ref err) if err.is_failure() => {
                debug!(
                    "{:?} failure handling input {:?}: {:?}",
                    connection_id, input, err
                );
            }

            _ => {}
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

    fn apply_action(
        &mut self,
        container: &mut Container,
        connection_id: ConnectionId,
        view_action: ConnectionViewAction,
    ) -> crate::errors::Result<()> {
        match &view_action {
            ConnectionViewAction::None => {}
            _ => info!("{:?} executing {:?}", connection_id, view_action),
        }

        match view_action {
            ConnectionViewAction::None => Ok(()),

            ConnectionViewAction::SwitchView(view) => {
                let state = self.connections.get_mut(&connection_id).unwrap();
                state.view = view;

                self.handle_view_welcome(container, connection_id)
            }

            ConnectionViewAction::Login(player_id) => {
                debug!("{:?} login in {:?}", connection_id, player_id);

                let state = self.connections.get_mut(&connection_id).unwrap();
                state.view = ConnectionView::Game;
                state.player_id = Some(player_id);

                self.connection_id_by_player_id
                    .insert(player_id, connection_id);

                self.handle_view_welcome(container, connection_id)
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

                self.handle_view_welcome(container, connection_id)
            }
        }
    }

    fn handle_view_welcome(
        &mut self,
        container: &mut Container,
        connection_id: ConnectionId,
    ) -> crate::errors::Result<()> {
        let state = self.connections.get(&connection_id).as_result_exception()?;

        match state.view {
            ConnectionView::Game => {
                let player_id = state.player_id.as_result_exception()?;
                let mob_id = container.players.get_mob(player_id).as_result_exception()?;
                crate::game::actions::look(container, mob_id)
            }

            ConnectionView::Login => {
                let msg = view_login::handle_welcome();
                self.server_outputs.push((connection_id, msg));
                Ok(())
            }

            ConnectionView::Admin => {
                let msg = view_admin::handle_welcome();
                self.server_outputs.push((connection_id, msg));
                Ok(())
            }
        }
    }

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

    /// Convert game output into connection output. Including convert the text into rich/plain
    ///  text depending of connection configuration.
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
                        debug!(
                            "{:?} output {:?}",
                            connection_id,
                            strip_rich_text(msg.clone())
                        );
                        let msg = process_rich_text(msg);
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
                        debug!(
                            "{:?} output {:?}",
                            connection_id,
                            strip_rich_text(msg.clone())
                        );
                        let msg = process_rich_text(msg.clone());
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

pub fn handle_request_get_objects(container: &Container) -> Result<Vec<ObjData>> {
    Ok(Loader::create_snapshot(container)?
        .objects
        .into_iter()
        .map(|(_k, v)| v)
        .collect())
}

pub fn handle_request_get_prefab(container: &Container, static_id: StaticId) -> Result<ObjData> {
    Ok(container
        .loader
        .get_prefab(static_id)
        .ok_or(Error::NotFoundStaticId(static_id))?
        .clone())
}

pub fn handle_request_get_prefabs(container: &Container) -> Result<Vec<ObjData>> {
    Ok(container.loader.list_prefabs().cloned().collect())
}

pub fn handle_request_get_object(container: &Container, id: ObjId) -> Result<ObjData> {
    if container.objects.exists(id) {
        let object = Loader::snapshot_obj(container, id)?;
        Ok(object)
    } else {
        Err(Error::NotFoundFailure)
    }
}

pub fn handle_request_remove_object(container: &mut Container, id: ObjId) -> Result<()> {
    container.remove(id);
    Ok(())
}

pub fn handle_request_remove_prefab(container: &mut Container, id: StaticId) -> Result<()> {
    container.loader.remove_prefab(id)?;
    Ok(())
}

pub fn handle_request_update_obj(container: &mut Container, data: ObjData) -> Result<()> {
    if data.id.is_none() || !container.objects.exists(data.id.unwrap().as_u32().into()) {
        return Err(Error::InvalidArgumentFailureStr(format!(
            "could not found object id {:?}",
            data.id
        )));
    }

    let obj_id = data.id.unwrap().as_u32().into();
    Loader::apply_data(container, obj_id, &data, &Default::default())
}

pub fn handle_request_add_obj(container: &mut Container, data: ObjData) -> Result<ObjId> {
    let obj_id = container.objects.create();
    let _ = Loader::apply_data(container, obj_id, &data, &Default::default())?;
    Ok(obj_id)
}

pub fn handle_request_add_prefab(container: &mut Container, data: ObjData) -> Result<StaticId> {
    if data.id.is_some() {
        Err(Error::InvalidArgumentFailureStr(
            "a new prefab should not have id".to_string(),
        ))
    } else {
        let id = container.loader.add_prefab(data)?;
        Ok(id)
    }
}

pub fn handle_request_update_prefab(container: &mut Container, data: ObjData) -> Result<()> {
    if data.id.is_none() {
        Err(Error::InvalidArgumentFailureStr(
            "a new prefab should not have id".to_string(),
        ))
    } else {
        container.loader.update_prefab(data)?;
        Ok(())
    }
}

pub fn handle_spawn_prefab(
    container: &mut Container,
    static_id: StaticId,
    parent_id: ObjId,
) -> Result<ObjId> {
    if container.loader.get_prefab(static_id).is_none() {
        return Err(Error::InvalidArgumentFailureStr(format!(
            "invalid argument, there is no prefab with id {:?}",
            static_id
        )));
    }

    if !container.objects.exists(parent_id) {
        return Err(Error::InvalidArgumentFailureStr(format!(
            "invalid argument, there is no object with id {:?}",
            parent_id
        )));
    }

    Loader::spawn_at(container, static_id, parent_id)
}

fn process_rich_text(mut msg: String) -> String {
    for mark in OMarker::list() {
        match mark {
            OMarker::Plain => {}
            OMarker::Literal => msg = msg.replace(mark.id(), &asciicolors::fg(45)),
            OMarker::Reset => msg = msg.replace(mark.id(), asciicolors::RESET),
            OMarker::Label => msg = msg.replace(mark.id(), &asciicolors::fg(226)),
        }
    }

    msg
}

fn strip_rich_text(mut msg: String) -> String {
    OMarker::strip(msg)
}
