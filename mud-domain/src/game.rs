use std::collections::{HashMap, HashSet};

use commons::*;
use container::Container;
use logs::*;

use crate::controller::Controller;
use crate::errors::*;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::room::RoomId;
use crate::game::system::{SystemCtx, Systems};

pub mod actions;
pub mod actions_admin;
pub mod actions_hire;
pub mod actions_items;
pub mod actions_ships;
pub mod actions_vendor;
pub mod astro_bodies;
pub mod avatars;
pub mod builder;
pub mod combat;
pub mod comm;
pub mod config;
pub mod container;
pub mod corpse;
pub mod domain;
pub mod equip;
pub mod hire;
pub mod inventory;
pub mod item;
pub mod labels;
pub mod loader;
pub mod location;
pub mod main_loop;
pub mod mob;
pub mod obj;
pub mod outputs;
pub mod ownership;
pub mod player;
pub mod pos;
pub mod prices;
pub mod random_rooms;
pub mod random_rooms_generator;
pub mod room;
pub mod rooms_zones;
pub mod ships;
pub mod space_utils;
pub mod spawn;
pub mod storages;
pub mod surfaces;
pub mod surfaces_object;
pub mod system;
pub mod tags;
pub mod template;
pub mod timer;
pub mod triggers;
pub mod vendors;
pub mod zone;

/// TODO: replace by buffer? looks a like of extra work keep this abstraction as reference
pub trait Outputs {
    /// For all mobs recursive inside the location
    fn broadcast_all(&mut self, exclude: Option<MobId>, location_id: LocationId, msg: String);
    /// For all mobs in current location
    fn broadcast(&mut self, exclude: Option<MobId>, location_id: LocationId, msg: String);
    /// Just to a specific mob
    fn private(&mut self, mob_id: MobId, msg: String);
}

/// Hold container and interface logic
pub struct Game {
    pub container: Container,
    controller: Controller,
    systems: Systems,
}

impl Game {
    pub fn new(mut container: Container) -> Self {
        let systems = Systems::new(&mut container);

        Game {
            container,
            controller: Controller::new(),
            systems,
        }
    }

    pub fn add_time(&mut self, delta_time: DeltaTime) {
        self.container.time.add(delta_time);
    }

    pub fn add_connection(&mut self, connection_id: ConnectionId) {
        self.controller
            .add_connection(&mut self.container, connection_id);
    }

    pub fn disconnect(&mut self, connection_id: ConnectionId) {
        self.controller
            .disconnect(&mut self.container, connection_id);
    }

    pub fn handle_input(&mut self, connection_id: ConnectionId, input: &str) {
        self.controller
            .handle_input(&mut self.container, connection_id, input);
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        crate::game::main_loop::tick(
            delta_time,
            &mut self.container,
            &mut self.systems,
            self.controller.get_outputs(),
        );
    }

    pub fn flush_outputs(&mut self) -> Vec<(ConnectionId, String)> {
        self.controller.flush_outputs(&self.container)
    }

    pub fn admin_kill_avatar_from_connection(&mut self, connection_id: ConnectionId) -> Result<()> {
        let player_id = self
            .controller
            .player_id_from_connection_id(connection_id)
            .as_result()?;
        let avatar_id = self.container.players.get(player_id).as_result()?;
        let mob_id = avatar_id.mob_id;
        actions_admin::force_kill(&mut self.container, self.controller.get_outputs(), mob_id)
    }
}

#[cfg(test)]
pub mod test {
    use crate::controller::OutputsBuffer;
    use crate::game::container::Container;
    use crate::game::system::Systems;
    use crate::game::{builder, main_loop};
    use commons::{DeltaTime, TotalTime};

    pub struct TestScenery {
        pub container: Container,
        pub systems: Systems,
        pub outputs: OutputsBuffer,
    }

    impl TestScenery {
        pub fn tick(&mut self, delta: f32) {
            main_loop::tick(
                DeltaTime(delta),
                &mut self.container,
                &mut self.systems,
                &mut self.outputs,
            );
        }
    }

    pub fn scenery() -> TestScenery {
        let mut container = Container::new();
        let systems = Systems::new(&mut container);
        let outputs = OutputsBuffer::new();

        TestScenery {
            container,
            systems,
            outputs,
        }
    }
}
