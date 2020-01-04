use std::collections::{HashMap, HashSet};

use commons::*;
use container::Container;
use logs::*;

use crate::controller::Controller;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::room::RoomId;

pub mod actions;
pub mod actions_admin;
pub mod actions_craft;
pub mod actions_items;
pub mod actions_vendor;
pub mod astro_bodies;
pub mod avatars;
pub mod builder;
pub mod combat;
pub mod comm;
pub mod config;
pub mod container;
pub mod corpse;
pub mod crafts;
pub mod crafts_system;
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
pub mod vendors;

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
    container: Container,
    controller: Controller,
}

impl Game {
    pub fn new(container: Container) -> Self {
        Game {
            container,
            controller: Controller::new(),
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
        self.container
            .tick(self.controller.get_outputs(), delta_time);
    }

    pub fn flush_outputs(&mut self) -> Vec<(ConnectionId, String)> {
        self.controller.flush_outputs(&self.container)
    }

//    pub fn run(&mut self, delta_time: DeltaTime, new_connections, disconnects, inputs) -> Vec<(ConnectionId, String)>{
//        self.add_time(delta_time);
//
//    }
}

#[cfg(test)]
pub mod test {
    use crate::game::builder;
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
        let room_id = builder::add_room(&mut container, "test_room");

        // TODO: remove hack when we use proper item builder
        let container_id = builder::add_item(&mut container, "container1", room_id);
        {
            let mut item = container.items.remove(container_id).unwrap();
            item.flags.is_stuck = true;
            item.flags.is_inventory = true;
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
