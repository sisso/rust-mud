use commons::*;
use std::collections::{HashMap, HashSet};

use container::Container;
use domain::*;
use item::*;
use item::ItemPrefabId;
use mob::*;
use mob::MobPrefabId;
use player::*;
use room::*;
use logs::*;

pub mod actions;
pub mod body;
pub mod comm;
pub mod container;
pub mod combat;
pub mod domain;
pub mod mob;
pub mod player;
pub mod room;
pub mod spawn;
pub mod view_main;
pub mod view_login;
pub mod item;
pub mod actions_items;
pub mod actions_admin;
pub mod loader;

pub struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

pub enum Output {
    Private {
        player_id: PlayerId,
        msg: String,
    },

    Room {
        /// player that originate the message, he is the only one will not receive the message
        player_id: Option<PlayerId>,
        room_id: RoomId,
        msg: String,
    },
}

impl Output {
    pub fn private(player_id: PlayerId, msg: String) -> Self {
        Output::Private {
            player_id,
            msg,
        }
    }

    pub fn room(player_id: PlayerId, room_id: RoomId, msg: String) -> Self {
        Output::Room {
            player_id: Some(player_id),
            room_id,
            msg,
        }
    }

    pub fn room_all(room_id: RoomId, msg: String) -> Self {
        Output::Room {
            player_id: None,
            room_id,
            msg,
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

    fn replace(&mut self) -> Vec<Output> {
        std::mem::replace(&mut self.list, vec![])
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
    pub outputs: &'a mut dyn Outputs,
}

pub struct Game {
    container: Container,
    connections: HashMap<ConnectionId, ConnectionState>,
    connection_id_by_player_id: HashMap<PlayerId, ConnectionId>,
    server_outputs: Vec<(ConnectionId, String)>,
    outputs: OutputsImpl,
    connections_with_input: HashSet<ConnectionId>,
}

impl Game {
    pub fn new(container: Container) -> Self {
        Game {
            container,
            connections: HashMap::new(),
            connection_id_by_player_id: HashMap::new(),
            server_outputs: vec![],
            outputs: OutputsImpl::new(),
            connections_with_input: Default::default(),
        }
    }

    pub fn add_connection(&mut self, connection_id: ConnectionId) {
        info!("gamecontroller - {:?} receive new player", connection_id);
        self.connections.insert(connection_id.clone(), ConnectionState {
            connection_id,
            player_id: None,
        });

        let msg = view_login::handle_welcome();

        self.server_outputs.push((connection_id, msg));
    }

    pub fn disconnect(&mut self, connection_id: ConnectionId) {
        let state = self.get_state(connection_id);

        if let Some(player_id) = state.player_id {
            info!("gamecontroller - {:?} removing player {:?}", connection_id, player_id);
            self.container.players.player_disconnect(player_id);
        } else {
            info!("gamecontroller - {:?} removing non logged player", connection_id);
        }

        self.connections.remove(&connection_id);
    }

    pub fn handle_input(&mut self, time: &GameTime, connection_id: ConnectionId, input: &str) {
        self.connections_with_input.insert(connection_id);

        let state = self.get_state(connection_id);

        if let Some(player_id) = state.player_id {
            debug!("gamecontroller - {:?} handling input '{}'", connection_id, input);
            view_main::handle(time, &mut self.container, &mut self.outputs, player_id, input);
        } else {
            debug!("gamecontroller - {:?} handling login '{}'", connection_id, input);
            let result = view_login::handle(&mut self.container, input);

            self.server_outputs.push((connection_id, result.msg));

            if let Some(player_id) = result.player_id {
                debug!("gamecontroller - {:?} login complete for {:?}", connection_id, player_id);

                self.set_state(ConnectionState {
                    connection_id,
                    player_id: Some(player_id),
                })
            }
        }
    }

    pub fn tick(&mut self, time: &GameTime) {
        let mut ctx = Ctx {
            time,
            container: &mut self.container,
            outputs: &mut self.outputs,
        };

        spawn::run(&mut ctx);
        mob::run_tick(&mut ctx);
        item::run_tick(&mut ctx);
    }

    pub fn get_outputs(&mut self) -> Vec<(ConnectionId, String)> {
        self.convert_to_connections_output();
        self.normalize_connection_outputs();

        // clear temporary
        self.connections_with_input.clear();

        // return outputs
        std::mem::replace(&mut self.server_outputs, vec![])
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        self.container.save(save);
//    }

    /// For each player that will receive output, append new line with cursor.
    ///
    /// If player send no input, append a new line before any output
    fn normalize_connection_outputs(&mut self) {
        let mut append_cursor_ids: Vec<ConnectionId> = vec![];
        let mut new_lines_ids: Vec<ConnectionId> = vec![];

        for (connection_id, _) in self.server_outputs.iter() {
            let player_id = self.player_id_from_connection_id(connection_id);

            match player_id {
                Some(_) if !append_cursor_ids.contains(connection_id) => {
                    append_cursor_ids.push(connection_id.clone());

                    // if player do not have newline because sent a input, append new line in start
                    if !self.connections_with_input.contains(connection_id) && !new_lines_ids.contains(&connection_id) {
                        new_lines_ids.push(connection_id.clone());
                    }
                }
                _ => {}
            }
        }

        for connection_id in new_lines_ids {
            self.server_outputs.insert(0, (connection_id, "\n".to_string()));
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
    fn convert_to_connections_output(&mut self) {
        let outputs = self.outputs.replace();

        for game_output in outputs {
            match game_output {
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
                        for connection_id in connections_id {
                            self.server_outputs.push((connection_id, msg.clone()));
                        }
                    } else {
                        debug!("game_controller - no players at room");
                    }
                }

                Output::Private { player_id, msg } => {
                    let connection_id = self.connection_id_from_player_id(player_id);
                    self.server_outputs.push((connection_id, msg));
                }
            }
        }
    }

    fn connection_id_from_player_id(&self, player_id: PlayerId) -> ConnectionId {
        *self.connection_id_by_player_id
            .get(&player_id)
            .expect(format!("could not found connection for {:?}", player_id).as_str())
    }

    fn get_state(&self, connection_id: ConnectionId) -> &ConnectionState {
        self.connections
            .get(&connection_id)
            .expect(format!("could not found connection for {:?}", connection_id).as_str())
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
                    let avatar = self.container.mobs.get(player.avatar_id);
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
        let state = self.connections.get(connection_id).expect(format!("could not found state for connection {:?}", connection_id).as_str());
        state.player_id
    }
}
