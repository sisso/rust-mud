extern crate mud_domain;

use commons::{ConnectionId, DeltaTime};
use logs::*;
use mud_domain::game::container::Container;
use mud_domain::game::prices::Money;
use mud_domain::game::{inventory_service, loader, Game, GameCfg};
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
        loader::Loader::load_folders(&mut container, &Path::new("../data/fantasy")).unwrap();
        TestScenery {
            game: Game::new(GameCfg::new(), container),
            connection_id: ConnectionId(0),
            timeout: 200,
        }
    }

    pub fn login(&mut self) {
        self.game.add_connection(self.connection_id);
        self.game.handle_input(self.connection_id, "player");
        self.wait_for("welcome back");
    }

    pub fn input(&mut self, s: &str) {
        self.game.handle_input(self.connection_id, s);
    }

    pub fn input_and_wait(&mut self, input: &str, expected: &str) {
        self.input(input);
        self.wait_for(expected);
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

    pub fn assert_output(&mut self, contains: Vec<&str>) {
        let outputs = self.take_outputs();
        if !check_output(&outputs, &contains, &vec![]) {
            assert_eq!(outputs.join("\n"), contains.join("\n"));
        }
    }

    pub fn wait_for(&mut self, contains: &str) -> Vec<String> {
        self.wait_until(vec![contains], vec![])
    }

    /// should have all contains, and if contain, should have no exclude
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
            self.input(command);
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

    pub fn give_money(&mut self, amount: u32) {
        let player_id = *self
            .game
            .container
            .players
            .list_players()
            .iter()
            .next()
            .unwrap();

        let mob_id = self.game.container.players.get_mob(player_id).unwrap();

        debug!("{:?} receive cheat {:?} of money", mob_id, amount);

        inventory_service::add_money(&mut self.game.container, mob_id, Money(amount)).unwrap();
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

#[test]
fn test_fantasy_kill_wolf_and_sell_meat() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_market(&mut scenery);
    // run the following lines multiple times can cause have multiple bodies
    from_market_to_forest(&mut scenery);
    kill_wolf_and_loot(&mut scenery);
    from_forest_to_market(&mut scenery);
    sell_meat(&mut scenery);
}

// // TODO: wolfs are not aggressive anymore
// // #[test]
// fn test_fantasy_wolf_are_aggressive() {
//     let mut scenery = TestScenery::new();
//     scenery.login();
//     from_village_to_forest(&mut scenery);
//     scenery.wait_for("wolf execute a attack");
// }

#[test]
fn test_fantasy_collect_money_should_be_merged() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_temple(&mut scenery);

    pick_money_from_chest(&mut scenery);
    assert_money(&mut scenery, 1);

    pick_money_from_chest(&mut scenery);
    assert_money(&mut scenery, 2);

    pick_money_from_chest(&mut scenery);
    assert_money(&mut scenery, 3);
}

#[test]
fn test_fantasy_steal_temple() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_temple(&mut scenery);
    for _ in 0..5 {
        pick_money_from_chest(&mut scenery);
    }
}

#[test]
fn test_fantasy_buy_weapon() {
    let mut scenery = TestScenery::new();
    scenery.login();
    scenery.give_money(1000);
    from_village_to_market(&mut scenery);
    buy_sword(&mut scenery);
    equip_sword(&mut scenery);
}

#[test]
fn test_fantasy_hire_mercenary_and_fight() {
    let mut scenery = TestScenery::new();
    scenery.login();
    scenery.give_money(100);
    // wait until mercenary spaw
    hire_mercenary(&mut scenery);
    from_village_to_forest(&mut scenery);
    // confirm mercenary have follow us
    scenery.input_and_wait("look", "mercenary");
    // wait for wolf
    scenery.repeat_command_until("look", "wolf");
    // kill wolf
    scenery.input("kill wolf");
    scenery.wait_until(vec!["mercenary", "attack", "wolf"], vec![]);
}

#[test]
fn test_fantasy_show_map() {
    let mut scenery = TestScenery::new();
    scenery.login();
    scenery.input("map");
    scenery.wait_until(vec!["Map", "**"], vec![]);
    // move around
    scenery.input_and_wait("w", "Temple");
    scenery.input_and_wait("e", "Village");
    scenery.input_and_wait("e", "Bar");
    scenery.input_and_wait("w", "Village");
    scenery.input("map");
    scenery.wait_until(vec!["Map", "00==**==02", "Bar", "Temple >"], vec![]);
    // assert_eq!(Vec::<String>::new(), scenery.take_outputs());
}

#[test]
fn test_fantasy_random_rooms() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_dungeons(&mut scenery);
    scenery.input("map");
    scenery.assert_output(vec!["Map", "02==**=="]);
}

#[test]
fn test_fantasy_player_respawn() {
    let mut scenery = TestScenery::new();
    scenery.login();
    from_village_to_temple(&mut scenery);
    scenery
        .game
        .container
        .admin_kill_avatar_from_connection(scenery.connection_id)
        .unwrap();
    scenery.assert_output(vec!["resurrect"]);
}

fn hire_mercenary(scenery: &mut TestScenery) {
    scenery.repeat_command_until("look", "mercenary");
    scenery.input("hire");
    scenery.wait_for("mercenary");
    scenery.input("hire mercenary");
    scenery.wait_for("mercenary hired");
}

fn sell_meat(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("- vendor");
    scenery.input("sell");
    scenery.wait_for("- meat");
    scenery.input("sell meat");
    scenery.wait_for("receive");
    scenery.input("inv");
    scenery.wait_for("- gold");
}

fn from_forest_to_market(scenery: &mut TestScenery) {
    scenery.input("n");
    scenery.wait_for("Market");
}

// TODO: test can be flacky when player is killed by the wolf, bad player
fn kill_wolf_and_loot(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_until(vec!["wolf"], vec!["corpse"]);

    // kill a wolf
    scenery.input("k wolf");
    scenery.wait_for("wolf corpse");
    scenery.input("examine corpse");
    scenery.wait_for("- meat");
    // collect loot
    scenery.input("get meat in corpse");
    scenery.wait_for("you pick");
    scenery.input("inv");
    scenery.wait_for("- meat");
}

fn from_village_to_market(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("Village");
    scenery.input("s");
    scenery.wait_for("Market");
    scenery.input("look");
    scenery.wait_for("vendor");
}

fn from_village_to_forest(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("Village");
    scenery.input("s");
    scenery.input("s");
    scenery.wait_for("forest");
}

fn from_village_to_dungeons(scenery: &mut TestScenery) {
    from_village_to_forest(scenery);
    scenery.input("e");
    scenery.wait_for("Dungeon");
}

fn from_market_to_forest(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("Market");
    scenery.input("s");
    scenery.wait_for("forest");
}

fn from_village_to_temple(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("Village");
    scenery.input("w");
    scenery.wait_for("Temple");
}

fn pick_money_from_chest(scenery: &mut TestScenery) {
    scenery.repeat_command_until("examine chest", "gold");
    scenery.input("get gold in chest");
    scenery.wait_for("pick");
}

// fn from_temple_to_market(scenery: &mut TestScenery) {
//     scenery.input_and_wait("e", "Village");
//     scenery.input_and_wait("s", "Market");
// }

fn buy_sword(scenery: &mut TestScenery) {
    scenery.input_and_wait("buy", "Short Sword");
    scenery.input_and_wait("buy short sword", "bought");
}

fn equip_sword(scenery: &mut TestScenery) {
    scenery.input_and_wait("equip short sword", "you equip");
}

fn assert_money(scenery: &mut TestScenery, expected: u32) {
    scenery.input("inv");
    if expected == 1 {
        scenery.wait_for("gold");
    } else {
        scenery.wait_for(&format!("gold x{}", expected));
    }
}
