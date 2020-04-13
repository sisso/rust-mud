use std::collections::{HashMap, HashSet};

use commons::*;
use container::Container;
use logs::*;

use crate::controller::Controller;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::room::RoomId;
use crate::game::system::{Systems, SystemCtx};

pub mod actions;
pub mod actions_admin;
pub mod actions_ships;
pub mod actions_items;
pub mod actions_vendor;
pub mod actions_hire;
pub mod astro_bodies;
pub mod avatars;
pub mod builder;
pub mod combat;
pub mod comm;
pub mod config;
pub mod container;
pub mod corpse;
pub mod ships;
pub mod domain;
pub mod equip;
pub mod inventory;
pub mod item;
pub mod labels;
pub mod loader;
pub mod location;
pub mod mob;
pub mod obj;
pub mod outputs;
pub mod player;
pub mod pos;
pub mod prices;
pub mod room;
pub mod space_utils;
pub mod spawn;
pub mod storages;
pub mod surfaces;
pub mod surfaces_object;
pub mod tags;
pub mod template;
pub mod timer;
pub mod triggers;
pub mod vendors;
pub mod system;
pub mod main_loop;
pub mod ownership;
pub mod zone;
pub mod rooms_zones;
pub mod hire;
pub mod random_rooms_generator;

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
        crate::game::main_loop::tick(delta_time, &mut self.container, &mut self.systems, self.controller.get_outputs());
    }

    pub fn flush_outputs(&mut self) -> Vec<(ConnectionId, String)> {
        self.controller.flush_outputs(&self.container)
    }
}

#[cfg(test)]
pub mod test {
    use crate::game::container::Container;
    use crate::game::system::Systems;
    use crate::game::{builder, main_loop};
    use commons::{TotalTime, DeltaTime};
    use crate::controller::OutputsBuffer;

    pub struct TestScenery {
        pub container: Container,
        pub systems: Systems,
        pub outputs: OutputsBuffer,
    }

    impl TestScenery {
        pub fn tick(&mut self, delta: f32) {
            main_loop::tick(DeltaTime(delta), &mut self.container, &mut self.systems, &mut self.outputs);
        }
    }

    pub fn scenery() -> TestScenery {
        let mut container = Container::new();
        let mut systems = Systems::new(&mut container);
        let mut outputs = OutputsBuffer::new();

        TestScenery {
            container,
            systems,
            outputs
        }
    }
}
