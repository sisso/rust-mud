use commons::*;
use std::collections::{HashMap, HashSet};
use logs::*;
use container::Container;
use crate::game::view_login::LoginResult;
use crate::game::room::RoomId;

pub mod obj;
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
pub mod input_handle_items;
pub mod location;
pub mod template;
pub mod avatars;
pub mod equip;
pub mod inventory;
pub mod tags;
pub mod builder;
pub mod labels;
pub mod config;

#[derive(Debug)]
pub struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

#[derive(Debug)]
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
    fn room_opt(&mut self, player_id: Option<PlayerId>, room_id: RoomId, msg: String);
    fn private_opt(&mut self, player_id: Option<PlayerId>, msg: String);
    fn private(&mut self, player_id: PlayerId, msg: String);
}

#[derive(Debug)]
pub struct OutputsBuffer {
    list: Vec<Output>
}

impl OutputsBuffer {
    pub fn new() -> Self {
        OutputsBuffer {
            list: vec![]
        }
    }

    pub fn take(&mut self) -> Vec<Output> {
        std::mem::replace(&mut self.list, vec![])
    }
}

impl Outputs for OutputsBuffer {
    fn room_all(&mut self, room_id: RoomId, msg: String) {
        self.list.push(Output::room_all(room_id, msg));
    }

    fn room(&mut self, player_id: PlayerId, room_id: RoomId, msg: String) {
        self.list.push(Output::room(player_id, room_id, msg));
    }

    fn room_opt(&mut self, player_id: Option<PlayerId>, room_id: RoomId, msg: String) {
        match player_id {
            Some(player_id) => self.room(player_id, room_id, msg),
            None => self.room_all(room_id, msg),
        }
    }
    fn private_opt(&mut self, player_id: Option<PlayerId>, msg: String) {
        match player_id {
            Some(player_id) => self.private(player_id, msg),
            None => {},
        }
    }

    fn private(&mut self, player_id: PlayerId, msg: String) {
        self.list.push(Output::private(player_id, msg));
    }
}

pub struct Game {
    container: Container,
    connections: HashMap<ConnectionId, ConnectionState>,
    connection_id_by_player_id: HashMap<PlayerId, ConnectionId>,
    server_outputs: Vec<(ConnectionId, String)>,
    outputs: OutputsBuffer,
    connections_with_input: HashSet<ConnectionId>,
}

// TODO: dilacerate this classe into mud-server
impl Game {
    pub fn new(container: Container) -> Self {
        Game {
            container,
            connections: HashMap::new(),
            connection_id_by_player_id: HashMap::new(),
            server_outputs: vec![],
            outputs: OutputsBuffer::new(),
            connections_with_input: Default::default(),
        }
    }

    pub fn add_time(&mut self, delta_time: DeltaTime) {
        self.container.time.add(delta_time);
    }

    pub fn add_connection(&mut self, connection_id: ConnectionId) {
        info!("{:?} receive connection", connection_id);
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
            info!("{:?} disconnecting player {:?}", connection_id, player_id);
            avatars::on_player_disconnect(&mut self.container, &mut self.outputs, player_id);
        } else {
            info!("{:?} disconnecting", connection_id);
        }

        self.connections.remove(&connection_id);
    }

    pub fn handle_input(&mut self, connection_id: ConnectionId, input: &str) {
        self.connections_with_input.insert(connection_id);

        let state = self.get_state(connection_id);

        if let Some(player_id) = state.player_id {
            debug!("{:?} handling input '{}'", connection_id, input);
            view_main::handle(&mut self.container, &mut self.outputs, player_id, input);
        } else {
            debug!("{:?} handling login '{}'", connection_id, input);
            match view_login::handle(&mut self.container, input) {
                LoginResult::Msg { msg } => {
                    self.server_outputs.push((connection_id, msg));
                },
                LoginResult::Login { login } => {
                    self.server_outputs.push((connection_id, view_login::on_login_success(login.as_str())));
                    let player_id = avatars::on_player_login(&mut self.container, &mut self.outputs, login.as_str());
                    debug!("{:?} login complete for {:?}", connection_id, player_id);
                    self.set_state(ConnectionState {
                        connection_id,
                        player_id: Some(player_id),
                    })
                }
            }
        }
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        self.container.tick(&mut self.outputs, delta_time);
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
        let outputs = self.outputs.take();

        for game_output in outputs {
            match game_output {
                Output::Room { player_id, room_id, msg } => {
                    debug!("game_controller - {:?}/{:?}: {}", player_id, room_id, msg);

                    let players_per_room = find_players_per_room(&self.container);

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
                                .flat_map(|i_player_id| self.connection_id_from_player_id(*i_player_id))
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
                    if let Some(connection_id) = self.connection_id_from_player_id(player_id) {
                        self.server_outputs.push((connection_id, msg));
                    }
                }
            }
        }
    }

    fn connection_id_from_player_id(&self, player_id: PlayerId) -> Option<ConnectionId> {
        self.connection_id_by_player_id
            .get(&player_id)
            .cloned()
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

    fn player_id_from_connection_id(&self, connection_id: &ConnectionId) -> Option<PlayerId> {
        self.connections.get(connection_id)
            .and_then(|i| i.player_id)
    }
}

pub fn find_players_per_room(container: &Container) -> HashMap<RoomId, Vec<PlayerId>> {
    let room_player: Vec<(RoomId, PlayerId)> =
        container.players.list_players()
            .into_iter()
            .flat_map(|player_id| {
                let player = container.players.get_player_by_id(player_id);
                container.locations.get(player.mob_id).map(|room_id| {
                    (room_id,player_id)
                })
            })
            .collect();

    // group_by
    let mut result: HashMap<RoomId, Vec<PlayerId>> = HashMap::new();
    for (room_id, player_id) in room_player {
        result.entry(room_id).or_insert(vec![]).push(player_id);
    }
    result
}

#[cfg(test)]
pub mod test {
    use super::builder;
    use crate::game::container::Container;
    use crate::game::room::RoomId;
    use crate::game::item::ItemId;
    use crate::game::mob::MobId;

    pub struct TestScenery {
        pub container: Container,
        pub room_id: RoomId,
        pub container_id: ItemId,
        pub item1_id: ItemId,
        pub item2_id: ItemId,
        pub mob_id: MobId,
    }

    pub fn setup() -> TestScenery {
        let mut container = Container::new();
        let room_id = builder::add_room(&mut container, "test_room", "");

        // TODO: remove hack when we use proper item builder
        let container_id = builder::add_item(&mut container, "container1", room_id);
        {
            let mut item = container.items.remove(container_id).unwrap();
            item.is_stuck = true;
            item.is_inventory = true;
            container.items.add(item);
        }

        let item1_id = builder::add_item(&mut container, "item1", room_id);
        let item2_id = builder::add_item(&mut container, "item2", container_id);

        let mob_id = builder::add_mob(&mut container, "mob", room_id);

        TestScenery {
            container,
            room_id,
            container_id,
            item1_id,
            item2_id,
            mob_id
        }
    }
}
