#![allow(dead_code, unused_variables, unused_imports)]

use container::*;
use controller::*;
use domain::*;
use mob::*;
use room::*;
use item::*;
use spawn::*;

use crate::server;
use crate::server_socket;
use std::ops::Add;

pub mod actions;
pub mod body;
pub mod comm;
pub mod controller;
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

use crate::utils::*;
use crate::utils::save::{SaveToFile, Save};

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
            damage: Damage { min: 2, max: 4 },
            pv: Pv { current: 10, max: 10, heal_rate: Seconds(1.0) },
            attack_calm_down: Seconds(1.0)
        },
        inventory: vec![],
    });

    container.mobs.add_prefab(MobPrefab {
        id: MOB_DRUNK,
        label: "Drunk".to_string(),
        attributes: Attributes {
            attack: 8,
            defense: 8,
            damage: Damage { min: 1, max: 2 },
            pv: Pv { current: 8, max: 8, heal_rate: Seconds(1.0) },
            attack_calm_down: Seconds(1.0),
        },
        inventory: vec![
            ITEM_DEF_COINS_2
        ],
    });
}

fn load_rooms(container: &mut Container) {
    let room_id_bar = RoomId(1);
    let room_id_florest = RoomId(2);

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
        exits: vec![(Dir::N, INITIAL_ROOM_ID), (Dir::S, room_id_florest)],
    };

    let room3 = Room {
        id: room_id_florest,
        label: "Florest".to_string(),
        desc: "A deep, ugly and dark florest.".to_string(),
        exits: vec![(Dir::N, room_id_bar)],
    };

    container.rooms.add(room1);
    container.rooms.add(room2);
    container.rooms.add(room3);
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

pub struct Game {
    server: Box<dyn server::Server>,
    game_time: GameTime,
    controller: GameController,
    save: Option<(String, TimeTrigger)>,
}

impl Game {
    pub fn new(server: Box<dyn server::Server>, save: Option<(String, Seconds)>) -> Self {
        let mut container: Container = Container::new();
        load(&mut container);

        Game {
            server,
            game_time: GameTime {
                tick: Tick(0),
                total: Seconds(0.0),
                delta: Seconds(0.1)
            },
            controller: GameController::new(container),
            save: save.map(|(file, seconds)| {
                (file, TimeTrigger::new(seconds, Seconds(0.0)))
            }),
        }
    }

    pub fn run(&mut self, delta: Seconds) {
        self.game_time.tick  = Tick(self.game_time.tick.0 + 1);
        self.game_time.total = self.game_time.total.add(delta);
        self.game_time.delta = delta;

        let result = self.server.run();
        
        let params = controller::GameControllerContext {
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs,
        };

        let outputs = self.controller.handle(self.game_time, params);
        self.server.append_output(outputs);

        if let Some((save_file, trigger)) = self.save.as_mut() {
            if trigger.check(self.game_time.total) {
                let save_file = format!("{}_{}.jsonp", save_file, self.game_time.tick.0);
                let mut save = SaveToFile::new(save_file.as_ref());
                self.controller.save(&mut save);
                save.close()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server;
    use crate::server_dummy;
    use crate::server_dummy::ServerDummy;
    use crate::game::Game;
    use crate::game::domain::*;
    use crate::utils::*;

    use std::cell::RefCell;
    use std::rc::Rc;
    use std::fs::File;

    struct TestGame {
        outputs: Rc<RefCell<Vec<String>>>,
        inputs: Rc<RefCell<Vec<String>>>,
        game: Game,
    }

    const DELTA: Seconds = Seconds(1.0);
    const SAFE: u32 = 10;
    const SAVE: &str = "/tmp/test";

    impl TestGame {
        pub fn new() -> Self {
            let server = ServerDummy::new();
            let outputs = server.get_outputs_pointer();
            let inputs = server.get_inputs_pointer();
            let _ = std::fs::remove_file(SAVE);
            let game = Game::new(Box::new(server), Some((SAVE.to_string(), DELTA)));

            TestGame {
                outputs,
                inputs,
                game
            }
        }

        pub fn look_and_wait_for(&mut self, expected: &str) {
            for i in 0..SAFE {
                self.input("look");
                self.run_tick();
                if self.get_outputs().iter().find(|i| i.contains(expected)).is_some() {
                    break;
                }
            }
        }

        pub fn wait_for(&mut self, expected: &str) {
            for i in 0..SAFE {
                self.run_tick();

                if self.get_outputs().iter().find(|i| i.contains(expected)).is_some() {
                    return;
                }
            }

            panic!(format!("failed to wait for expected '{}' after {} ticks", expected, SAFE));
        }

        pub fn run_tick(&mut self) {
            self.game.run(DELTA);
        }

        pub fn get_outputs(&self) -> Vec<String> {
            let outputs = self.outputs.replace(vec![]);
            debug!("testserver.get_outputs - {:?}", outputs);
            outputs
        }

        pub fn input(&mut self, input: &str) {
            self.inputs.borrow_mut().push(input.to_string());
        }
    }

    #[test]
    fn kill_something() {
        let mut g = TestGame::new();
        do_login(&mut g);
        do_move_to_bar_wait_for_drunk(&mut g);
        g.input("kill Drunk");
        g.wait_for("killed");
        g.input("examine body");
        g.run_tick();
        let _ = g.get_outputs();
        g.input("pick body coins");
        g.run_tick();
        let _ = g.get_outputs();
        g.input("stats");
        g.run_tick();
        assert!(g.get_outputs().iter().find(|msg| msg.contains("- coins (2)")).is_some());
    }

    #[test]
    fn admin_kill_test() {
        let mut g = TestGame::new();
        do_login(&mut g);
        g.input("admin suicide");
        g.run_tick();
        g.wait_for("you have resurrected!");
        do_look_main_room(&mut g);
        g.input("rest");
        g.wait_for("sit and rest");
        g.wait_for("healing");
        g.wait_for("healed");
        g.input("stand");
        do_move_to_bar_wait_for_drunk(&mut g);
    }

    fn do_move_to_bar_wait_for_drunk(g: &mut TestGame) {
        g.input("s");
        g.wait_for("Bar");
        g.look_and_wait_for("Drunk");
    }

    fn do_login(g: &mut TestGame) {
        g.wait_for("Welcome to MUD");
        g.input("sisso");
        g.wait_for("welcome sisso");
        do_look_main_room(g);
    }

    fn do_look_main_room(g: &mut TestGame) {
        g.input("look");
        g.wait_for("Main Room");
    }
}
