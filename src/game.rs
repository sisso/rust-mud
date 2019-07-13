#![allow(dead_code, unused_variables, unused_imports)]

use container::*;
use controller::*;
use domain::*;
use mob::*;
use room::*;
use item::*;
use spawn::*;

use crate::server;
use crate::server::SocketServer;
use std::ops::Add;

mod actions;
mod body;
mod comm;
mod controller;
mod container;
mod combat;
mod domain;
mod mob;
mod player;
mod room;
mod spawn;
mod view_main;
mod view_login;
mod item;
mod actions_items;

const INITIAL_ROOM_ID: RoomId = RoomId(0);

const MOB_PLAYER: MobPrefabId = MobPrefabId(0);
const MOB_DRUNK: MobPrefabId  = MobPrefabId(1);

const ITEM_DEF_COINS_2: ItemPrefabId = ItemPrefabId(0);

fn load_items_prefabs(container: &mut Container) {
    container.items.add_prefab(ItemPrefab {
        id: ITEM_DEF_COINS_2,
        typ: ITEM_TYPE_GOLD,
        amount: 2,
        label: "coins".to_string(),
    });
}

fn load_mobs_prefabs(container: &mut Container) {
    container.mobs.add_prefab(MobPrefab {
        id: MOB_PLAYER,
        label: "Avatar".to_string(),
        attributes: Attributes {
            attack: 12,
            defense: 12,
            damage: Damage { min: 2, max: 4, calm_down: Seconds(1.0) },
            pv: Pv { current: 10, max: 10 },
        },
        inventory: vec![],
    });

    container.mobs.add_prefab(MobPrefab {
        id: MOB_DRUNK,
        label: "Drunk".to_string(),
        attributes: Attributes {
            attack: 8,
            defense: 8,
            damage: Damage { min: 1, max: 2, calm_down: Seconds(1.0) },
            pv: Pv { current: 8, max: 8 },
        },
        inventory: vec![
            ITEM_DEF_COINS_2
        ],
    });
}

fn load_rooms(container: &mut Container) {
    let room_id_bar = RoomId(1);

    let room1 = Room {
        id: INITIAL_ROOM_ID,
        label: "Main Room".to_string(),
        desc: "Main room where people born".to_string(),
        exits: vec![(Dir::S, room_id_bar)],
    };

    let room2 = Room {
        id: room_id_bar,
        label: "Bar".to_string(),
        desc: "Where we relief our duties".to_string(),
        exits: vec![(Dir::N, INITIAL_ROOM_ID)],
    };

    container.rooms.add(room1);
    container.rooms.add(room2);
}

fn load_spawns(container: &mut Container) {
    container.add_spawn(Spawn {
        id: SpawnId(0),
        room_id: RoomId(1),
        max: 1,
        delay: SpawnDelay {
            min: Seconds(5.0),
            max: Seconds(20.0),
        },
        prefab_id: MOB_DRUNK,
        next: Some(Seconds(1.0)),
        mobs_id: vec![],
    });
}

fn load(container: &mut Container) {
    load_items_prefabs(container);
    load_mobs_prefabs(container);
    load_rooms(container);
    load_spawns(container);
}

trait GameServer {
    fn run(&mut self, pending_outputs: Vec<server::Output>) -> server::LoopResult;
}

struct SocketGameServer {
    server: server::SocketServer,
}

impl SocketGameServer {
    fn new() -> Self {
        let mut server = server::SocketServer::new();
        server.start();

        SocketGameServer {
            server
        }
    }
}

impl GameServer for SocketGameServer {
    fn run(&mut self, pending_outputs: Vec<server::Output>) -> server::LoopResult {
        self.server.run(pending_outputs)
    }
}

pub fn run() {
    let server = SocketGameServer::new();
    let mut game = Game::new(Box::new(server));

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        game.run(Seconds(0.1));
    }
}

struct Game {
    server: Box<GameServer>,
    game_time: GameTime,
    controller: GameController,
    pending_outputs: Option<Vec<server::Output>>,
}

impl Game {
    pub fn new(server: Box<GameServer>) -> Self {
        let mut container: Container = Container::new();
        load(&mut container);

        Game {
            server: server,
            game_time: GameTime {
                tick: Tick(0),
                total: Seconds(0.0),
                delta: Seconds(0.1)
            },
            controller: GameController::new(container),
            pending_outputs: None
        }
    }

    pub fn run(&mut self, delta: Seconds) {
        self.game_time.tick  = Tick(self.game_time.tick.0 + 1);
        self.game_time.total = self.game_time.total.add(delta);
        self.game_time.delta = delta;
        let outputs = self.pending_outputs.take().unwrap_or(vec![]);

        let result = self.server.run(outputs);
        
        let params = controller::GameControllerContext {
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs,
        };

        self.pending_outputs = Some(self.controller.handle(self.game_time, params));
    }
}

#[cfg(test)]
mod tests {
    use super::GameServer;

    struct StubGameServer {

    }

    impl StubGameServer{
        fn new() -> Self {
            StubGameServer {

            }
        }
    }

    impl GameServer for StubGameServer {
        fn run(&mut self, pending_outputs: Vec<crate::server::Output>) -> crate::server::LoopResult {
            unimplemented!()
        }
    }

    #[test]
    fn kill_something() {
        let server = StubGameServer::new();
    }
}
