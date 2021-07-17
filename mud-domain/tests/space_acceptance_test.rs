extern crate mud_domain;

use commons::{ConnectionId, DeltaTime};
use logs::*;
use mud_domain::game::container::Container;
use mud_domain::game::{loader, Game, GameCfg};
use std::path::Path;

pub struct TestScenery {
    pub game: Game,
    pub connection_id: ConnectionId,
}

impl TestScenery {
    pub fn new_active() -> Self {
        let mut container = Container::new();
        loader::Loader::load_folders(&mut container, &Path::new("../data/space")).unwrap();
        TestScenery {
            game: Game::new(GameCfg::new(), container),
            connection_id: ConnectionId(0),
        }
    }

    pub fn new_from_file(file: &str) -> Self {
        let mut container = Container::new();
        loader::Loader::load_hocon_file(&mut container, file).unwrap();
        TestScenery {
            game: Game::new(GameCfg::new(), container),
            connection_id: ConnectionId(0),
        }
    }

    pub fn new_sectors_with_jump() -> Self {
        TestScenery::new_active()
    }

    pub fn new_landed_with_ship() -> Self {
        TestScenery::new_active()
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

    pub fn eventually(&mut self, command: &str, expected: &str) -> Vec<String> {
        let mut buffer = vec![];

        for _ in 0..100 {
            self.send_input(command);
            self.tick();

            let outputs = self.take_outputs();
            if check_output(&outputs, &vec![expected], &vec![]) {
                return outputs;
            } else {
                buffer.extend(outputs);

                // wait a bit
                for _ in 0..10 {
                    self.tick();
                }
            }
        }

        panic!(format!(
            "timeout waiting for {:?}, output received\n{}",
            expected,
            buffer.join("\n")
        ));
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

/// should have all contains, and if contain, should have no exclude
fn check_output(outputs: &Vec<String>, contains: &Vec<&str>, exclude: &Vec<&str>) -> bool {
    for expected in contains {
        match outputs.iter().find(|s| s.contains(expected)) {
            Some(line) => {
                for not_expected in exclude {
                    if line.contains(not_expected) {
                        return false;
                    }
                }
            }
            None => return false,
        }
    }

    true
}

struct SceneryMin {
    base: TestScenery,
}

impl SceneryMin {
    pub fn new() -> Self {
        let base = TestScenery::new_from_file("../data/tests/scenery_space_min.conf");
        SceneryMin { base }
    }

    fn move_to_cockpit(&mut self) {
        self.base.send_input("look");
        self.base.wait_for("transport");
    }

    fn launch(&mut self) {
        todo!()
    }
    fn land(&mut self) {
        todo!()
    }
    fn move_out_ship(&mut self) {
        todo!()
    }
}

// should replace test_fly_to_and_land
#[test]
fn test_launch_and_land() {
    let mut s = SceneryMin::new();
    s.base.login();
    s.move_to_cockpit();
    s.launch();
    s.land();
}

#[test]
fn test_fly_to_and_land() {
    let mut scenery = TestScenery::new_landed_with_ship();
    scenery.login();

    move_to_space(&mut scenery);
    fly_and_land_at_jumanji(&mut scenery);
}

#[test]
fn test_launch_land_and_launch() {
    let mut scenery = TestScenery::new_landed_with_ship();
    scenery.login();

    move_to_space(&mut scenery);
    land_at(&mut scenery, "Landing Pad");
    launch_ship(&mut scenery);
}

#[test]
fn test_jump_to_sector_2() {
    let mut scenery = TestScenery::new_sectors_with_jump();
    let scenery = &mut scenery;
    scenery.login();
    move_to_space(scenery);
    jump_to_sector_2(scenery);
}

fn fly_and_land_at_jumanji(scenery: &mut TestScenery) {
    scenery.send_input("move");
    scenery.wait_for("Jumanji");

    scenery.send_input("move jumanji");
    scenery.wait_for("command accepted");
    assert_ship_in_orbit_sol(scenery);
    scenery.wait_for("command complete");

    land_at(scenery, "Landing Pad");
}

fn move_to_space(scenery: &mut TestScenery) {
    go_to_landing_pad(scenery);
    enter_ship_and_move_to_cockpit(scenery);
    launch_ship(scenery);
}

fn land_at(scenery: &mut TestScenery, place: &str) {
    scenery.send_input("land");
    scenery.wait_for(place);

    scenery.send_input(&format!("land {}", place));
    scenery.wait_for("landing complete");
}

fn launch_ship(scenery: &mut TestScenery) {
    scenery.send_input("launch");
    scenery.wait_for("launch complete");

    scenery.send_input("sm");
    scenery.wait_for("Dune");
}

fn enter_ship_and_move_to_cockpit(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("Shuttle");

    scenery.send_input("enter shuttle");
    scenery.wait_for("you enter in");

    scenery.send_input("look");
    scenery.wait_for("Cockpit");

    scenery.send_input("out");
    scenery.wait_for("Landing Pad");

    scenery.send_input("enter shuttle");
    scenery.wait_for("you enter in");

    scenery.send_input("look");
    scenery.wait_for("Cockpit");
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

fn assert_ship_in_orbit_sol(scenery: &mut TestScenery) {
    scenery.eventually("sm", "\n- Shuttle");
}
