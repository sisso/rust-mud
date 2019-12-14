use crate::game::location::LocationId;
use crate::game::room::RoomId;
use crate::game::view_login::LoginResult;
use commons::*;
use container::Container;
use logs::*;
use std::collections::{HashMap, HashSet};
use crate::game::mob::MobId;

pub mod actions;
pub mod actions_admin;
pub mod actions_craft;
pub mod actions_items;
pub mod avatars;
pub mod body;
pub mod builder;
pub mod combat;
pub mod comm;
pub mod config;
pub mod container;
pub mod crafts;
pub mod crafts_system;
pub mod domain;
pub mod equip;
pub mod input_handle_items;
pub mod input_handle_space;
pub mod inventory;
pub mod item;
pub mod labels;
pub mod loader;
pub mod location;
pub mod mob;
pub mod obj;
pub mod planets;
pub mod player;
pub mod pos;
pub mod room;
pub mod space_utils;
pub mod spawn;
pub mod storages;
pub mod surfaces;
pub mod surfaces_object;
pub mod tags;
pub mod template;
pub mod view_login;
pub mod view_main;

#[derive(Debug)]
pub struct ConnectionState {
    pub connection_id: ConnectionId,
    pub player_id: Option<PlayerId>,
}

#[derive(Debug)]
pub enum Output {
    Private {
        mob_id: MobId,
        msg: String,
    },

    Broadcast {
        /// usually the mob that originate the message
        exclude: Option<MobId>,
        /// RoomId or ZoneId, all children mobs will receive the message
        location_id: LocationId,
        msg: String,
    },
}

pub trait Outputs {
    fn broadcast(&mut self, exclude: Option<MobId>, location_id: LocationId, msg: String);
    fn private(&mut self, mob_id: MobId, msg: String);
}

#[derive(Debug)]
pub struct OutputsBuffer {
    list: Vec<Output>,
}

impl OutputsBuffer {
    pub fn new() -> Self {
        OutputsBuffer { list: vec![] }
    }

    pub fn take(&mut self) -> Vec<Output> {
        std::mem::replace(&mut self.list, vec![])
    }
}

impl Outputs for OutputsBuffer {
    fn broadcast(&mut self, exclude: Option<ObjId>, location_id: ObjId, msg: String) {
        self.list.push(Output::Broadcast {
            exclude,
            location_id,
            msg
        })
    }

    fn private(&mut self, mob_id: MobId, msg: String) {
        self.list.push(Output::Private{
            mob_id,
            msg
        })
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
            let mob_id = self.container.players.get(player_id).mob_id;
            view_main::handle(&mut self.container, &mut self.outputs, mob_id, input);
        } else {
            debug!("{:?} handling login '{}'", connection_id, input);
            match view_login::handle(&mut self.container, input) {
                LoginResult::Msg { msg } => {
                    self.server_outputs.push((connection_id, msg));
                }
                LoginResult::Login { login } => {
                    self.server_outputs
                        .push((connection_id, view_login::on_login_success(login.as_str())));
                    let player_id = avatars::on_player_login(
                        &mut self.container,
                        &mut self.outputs,
                        login.as_str(),
                    );
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
    fn convert_to_connections_output(&mut self) {
        let outputs = self.outputs.take();

        for game_output in outputs {
            match game_output {
                Output::Private { mob_id, msg } => {
                    let connection_id= self.container.players.find_from_mob(mob_id)
                        .and_then(|player_id| self.zip_connection_id_from_player_id(player_id));

                    if let Some((player_id, connection_id)) = connection_id {
                            debug!("{:?} - {}", player_id, msg);
                            self.server_outputs.push((connection_id, msg));
                        }
                }

                Output::Broadcast {
                    exclude,
                    location_id,
                    msg
                } => {
                    let exclude_player = exclude.and_then(|mob_id| self.container.players.find_from_mob(mob_id));

                    let connections: Vec<(PlayerId, ConnectionId)> =
                        avatars::find_deep_all_players_in(&self.container, location_id)
                            .iter()
                            .filter(|&&player_id| Some(player_id) != exclude)
                            .flat_map(|&player_id| self.zip_connection_id_from_player_id(player_id))
                            .collect();

                    for (player_id, connection_id) in connections {
                        debug!("{:?} - {}", player_id, msg);
                        self.server_outputs.push((connection_id, msg.clone()))
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

pub fn find_players_per_room(container: &Container) -> HashMap<RoomId, Vec<PlayerId>> {
    let room_player: Vec<(RoomId, PlayerId)> = container
        .players
        .list_players()
        .into_iter()
        .flat_map(|player_id| {
            let player = container.players.get(player_id);
            container
                .locations
                .get(player.mob_id)
                .map(|room_id| (room_id, player_id))
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
    use crate::game::item::ItemId;
    use crate::game::mob::MobId;
    use crate::game::room::RoomId;

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
            mob_id,
        }
    }
}
