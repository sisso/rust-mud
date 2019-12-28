extern crate mud_domain;

use commons::{ConnectionId, DeltaTime};
use mud_domain::game::container::Container;
use mud_domain::game::{loader, Game};
use std::path::Path;

pub struct TestScenery {
    pub game: Game,
    pub connection_id: ConnectionId,
    pub timeout: u32,
}

impl TestScenery {
    pub fn new() -> Self {
        let mut container = Container::new();
        // loader::scenery_space::load(&mut container);
        loader::Loader::load_folder(&mut container, &Path::new("../data/fantasy")).unwrap();
        TestScenery {
            game: Game::new(container),
            connection_id: ConnectionId(0),
            timeout: 200
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

    pub fn wait_for(&mut self, contains: &str) -> Vec<String> {
        self.wait_until(vec![contains], vec![])
    }

    pub fn wait_until(&mut self, contains: Vec<&str>, exclude: Vec<&str>) -> Vec<String> {
        for _ in 0..self.timeout {
            let outputs = self.take_outputs();

            if check_output(&outputs, &contains, &exclude) {
                return outputs;
            } else {
                self.tick();
            }
        }

        panic!(format!("timeout waiting for {:?}", contains));
    }

    pub fn tick(&mut self) {
        self.game.tick(DeltaTime(0.5));
    }

    pub fn repeat_command_until(&mut self, command: &str, expected: &str) -> Vec<String> {
        for _ in 0..self.timeout {
            self.send_input(command);
            self.tick();

            let outputs = self.take_outputs();
            if check_output(&outputs, &vec![expected], &vec![]) {
                return outputs;
            } else {
                // wait some seconds
                for _ in 0..10 {
                    self.tick();
                }
            }
        }

        panic!(format!("timeout waiting for {:?}", expected));
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
            },
            None => return false,
        }
    }

    true
}

#[test]
fn test_fantasy_kill_wolf_and_sell_meat() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_market(&mut scenery);
    // run the following lines multiple times can cause have multiple bodies
    from_market_to_florest(&mut scenery);
    kill_wolf_and_loot(&mut scenery);
    from_florest_to_market(&mut scenery);
    sell_meat(&mut scenery);
}

#[test]
fn test_fantasy_collect_money_should_be_merged() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_temple(&mut scenery);
    pick_money_from_chest(&mut scenery);
    pick_money_from_chest(&mut scenery);
    assert_money_gt_0(&mut scenery);
}

#[test]
fn test_fantasy_steal_temple_and_buy_weapon() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_temple(&mut scenery);
    pick_money_from_chest(&mut scenery);
    from_temple_to_market(&mut scenery);
    buy_sword(&mut scenery);
    equip_sword(&mut scenery);
}

fn sell_meat(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("- vendor");
    scenery.send_input("list");
    scenery.wait_for("- meat");
    scenery.send_input("sell meat");
    scenery.wait_for("receive");
    scenery.send_input("inv");
    scenery.wait_for("- gold");
}

fn from_florest_to_market(scenery: &mut TestScenery) {
    scenery.send_input("n");
    scenery.wait_for("Market");
}

fn kill_wolf_and_loot(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_until(vec!["wolf"], vec!["corpse"]);

    // kill a wolf
    scenery.send_input("k wolf");
    scenery.wait_for("wolf corpse");
    scenery.send_input("examine corpse");
    scenery.wait_for("- meat");
    // collect loot
    scenery.send_input("get meat in corpse");
    scenery.wait_for("you pick");
    scenery.send_input("inv");
    scenery.wait_for("- meat");
}

fn from_village_to_market(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("Village");
    scenery.send_input("s");
    scenery.wait_for("Market");
}

fn from_market_to_florest(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("Market");
    scenery.send_input("s");
    scenery.wait_for("Florest");
}

fn from_village_to_temple(scenery: &mut TestScenery) {
    scenery.send_input("look");
    scenery.wait_for("Village");
    scenery.send_input("w");
    scenery.wait_for("Temple");
}

fn pick_money_from_chest(scenery: &mut TestScenery) {
    // pick up 3 gold coins
    for _ in 0..3 {
        scenery.repeat_command_until("examine chest", "gold");
        scenery.send_input("get gold in chest");
        scenery.wait_for("pick");
    }

    scenery.send_input("inv");
    scenery.wait_for("gold x3");
}

fn from_temple_to_market(scenery: &mut TestScenery) {
    unimplemented!()
}

fn buy_sword(scenery: &mut TestScenery) {
    unimplemented!()
}

fn equip_sword(scenery: &mut TestScenery) {
    unimplemented!()
}

fn assert_money_gt_0(scenery: &mut TestScenery) {
    unimplemented!()
}
