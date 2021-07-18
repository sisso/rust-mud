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
    pub fn new(files: &Vec<&str>) -> Self {
        let mut container = Container::new();
        loader::Loader::load_hocon_files(&mut container, files).unwrap();
        TestScenery {
            game: Game::new(GameCfg::new(), container),
            connection_id: ConnectionId(0),
        }
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

    pub fn send_wait(&mut self, command: &str, expected: &str) -> Vec<String> {
        self.send_input(command);
        self.wait_for(expected)
    }

    pub fn eventually(&mut self, command: &str, expected: &str) -> Vec<String> {
        let mut buffer = vec![];

        for _ in 0..10 {
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
        let base = TestScenery::new(&vec!["../data/tests/scenery_space_min.conf"]);
        let mut s = SceneryMin { base };
        s.base.login();
        s
    }

    fn enter_ship_and_move_to_bridge(&mut self) {
        self.base.send_input("look");
        self.base.wait_for("transport");
        self.base.send_input("enter transport");
        self.base.wait_for("you enter");
        self.base.send_input("n");
        self.base.wait_for("you move");
        self.base.send_input("n");
        self.base.wait_for("bridge");
    }

    fn launch(&mut self) {
        self.base.send_input("launch");
        self.base.wait_for("launch complete");
    }
    fn land(&mut self) {
        self.base.send_input("land");
        self.base.wait_for("landing zone");
        self.base.send_input("land landing zone");
        self.base.wait_for("landing complete");
    }
    fn move_out_ship(&mut self) {
        self.base.send_input("s");
        self.base.send_input("s");
        self.base.send_input("exit");
        self.base.wait_for("landing zone");
    }
    fn move_to_sector2(&mut self) {
        self.base.send_input("move");
        self.base.wait_for("jump sector2");
        self.base.send_input("move jump sector2");
        self.base.wait_for("command complete");
        self.base.send_input("sm");
        self.base.send_input("jump");
        self.base.wait_for("jump complete");
        self.base.send_input("sm");
    }
    fn move_to_planet2(&mut self) {
        self.base.send_input("move");
        self.base.send_input("move planet2");
        self.base.wait_for("command complete");
    }
    fn assert_ship_in_orbit_sol(&mut self) {
        self.base.eventually("sm", "- transport");
    }
}

#[test]
fn test_launch_and_land() {
    let mut s = SceneryMin::new();
    s.enter_ship_and_move_to_bridge();
    s.launch();
    s.assert_ship_in_orbit_sol();
    s.move_to_sector2();
    s.move_to_planet2();
    s.land();
}

#[test]
fn test_mine_ore() {
    let mut s = TestScenery::new(&vec![
        "../data/tests/scenery_min.conf",
        "../data/tests/scenery_min_extractable.conf",
    ]);
    s.login();
    s.send_wait("look", "ore deposit");
    s.send_wait("extract", "you start");
    s.wait_for("you extract");
    s.send_wait("inv", "metal ore");
}
