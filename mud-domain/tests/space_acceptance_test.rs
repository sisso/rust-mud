extern crate mud_domain;

use commons::{ConnectionId, DeltaTime};
use mud_domain::game::container::Container;
use mud_domain::game::{loader, Game};
use std::path::Path;

pub struct TestScenery {
    pub game: Game,
    pub connection_id: ConnectionId,
}

impl TestScenery {
    pub fn new() -> Self {
        let mut container = Container::new();
        // loader::scenery_space::load(&mut container);
        loader::Loader::load_folder(&mut container, &Path::new("../data/space")).unwrap();
        TestScenery {
            game: Game::new(container),
            connection_id: ConnectionId(0),
        }
    }

    pub fn new_sectors_with_jump() -> Self {
        TestScenery::new()
    }

    pub fn new_landed_with_ship() -> Self {
        TestScenery::new()
    }

    pub fn login(&mut self) {
        self.game.add_connection(self.connection_id);
        self.game.handle_input(self.connection_id, "player");
        self.wait_for("welcome back");
    }

    pub fn send_input(&mut self, s: &str) {
        self.game.handle_input(self.connection_id, s);
    }

    pub fn take_outputs(&mut self) -> Vec<String> {
        self.game
            .flush_outputs()
            .into_iter()
            .filter_map(|(id, msg)| {
                if id == self.connection_id {
                    Some(msg)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn wait_for(&mut self, contains: &str) -> Vec<String> {
        let mut buffer = vec![];

        for _ in 0..100 {
            let outputs = self.take_outputs();
            let found = outputs.iter().find(|msg| msg.contains(contains));

            if found.is_some() {
                return outputs;
            } else {
                buffer.extend(outputs);
                self.tick();
            }
        }

        panic!(format!(
            "timeout waiting for {:?}, outputs received\n{}",
            contains,
            buffer.join("\n")
        ));
    }

    pub fn tick(&mut self) {
        self.game.tick(DeltaTime(0.5));
    }
}

fn assert_outputs_contains(outputs: &Vec<String>, msg: &str) {
    for i in outputs {
        if i.contains(msg) {
            return;
        }
    }

    panic!("can not find {:?} in outputs {:?}", msg, outputs);
}

#[test]
fn test_sectormap() {
    let mut scenery = TestScenery::new_landed_with_ship();
    scenery.login();

    move_to_space(&mut scenery);
    fly_and_land_at_asteroid(&mut scenery);
}

#[test]
fn test_jump_to_sector_2() {
    let mut scenery = TestScenery::new_sectors_with_jump();
    let scenery = &mut scenery;
    scenery.login();
    move_to_space(scenery);
    jump_to_sector_2(scenery);
}

fn fly_and_land_at_asteroid(scenery: &mut TestScenery) {
    scenery.send_input("move");
    scenery.wait_for("Asteroid Field");

    scenery.send_input("move asteroid");
    scenery.wait_for("command accepted");
    scenery.wait_for("command complete");

    scenery.send_input("land");
    scenery.wait_for("Asteroid");

    scenery.send_input("land asteroid");
    scenery.wait_for("landing complete");
}

fn move_to_space(scenery: &mut TestScenery) {
    go_to_landing_pad(scenery);
    enter_ship_and_move_to_cockpit(scenery);
    launch_ship(scenery);
}

fn launch_ship(scenery: &mut TestScenery) {
    scenery.send_input("launch");
    scenery.wait_for("launch complete");

    scenery.send_input("sm");
    scenery.wait_for("Dune");
}

fn enter_ship_and_move_to_cockpit(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("Light Transport");

    scenery.send_input("enter transport");
    scenery.wait_for("you enter in");

    scenery.send_input("out");
    scenery.wait_for("Landing Pad");

    scenery.send_input("enter transport");
    scenery.wait_for("you enter in");

    scenery.send_input("n");
    scenery.send_input("n");
    scenery.wait_for("Bridge");
}

fn go_to_landing_pad(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("Palace");

    scenery.send_input("s");
    scenery.wait_for("Landing Pad");
}
fn jump_to_sector_2(scenery: &mut TestScenery) {
    scenery.send_input("move Jump point");
    scenery.wait_for("command accepted");
    scenery.wait_for("command complete");
    scenery.send_input("jump");
    scenery.wait_for("jump complete");
    scenery.send_input("sm");
    scenery.wait_for("Sector 2");
}
