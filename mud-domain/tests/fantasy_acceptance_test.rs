extern crate mud_domain;

use commons::{ConnectionId, DeltaTime, ObjId};

use mud_domain::game::builder;
use mud_domain::game::container::Container;

use mud_domain::game::loader::dto::StaticId;
use mud_domain::game::loader::Loader;
use mud_domain::game::outputs::OMarker;
use mud_domain::game::prices::Money;
use mud_domain::game::{inventory_service, loader, Game, GameCfg};
use std::path::Path;

pub struct TestScenery {
    pub game: Game,
    pub connection_id: ConnectionId,
    pub timeout: u32,
}

impl TestScenery {
    pub fn new(container: Container) -> Self {
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

        panic!("timeout waiting for {:?}", contains);
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

        panic!("timeout waiting for {:?}", expected);
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

        log::debug!("{:?} receive cheat {:?} of money", mob_id, amount);

        inventory_service::add_money(&mut self.game.container, mob_id, Money(amount)).unwrap();
    }
}

/// should have all contains, and if contain, should have no exclude
fn check_output(outputs: &Vec<String>, contains: &Vec<&str>, exclude: &Vec<&str>) -> bool {
    for expected in contains {
        match outputs.iter().find(|s| s.contains(expected)) {
            Some(line) => {
                let line = OMarker::strip(line.clone());

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

fn load_city_forest_wolf_vendor(container: &mut Container) {
    loader::Loader::load_hocon_file(container, "../data/tests/scenery_fantasy_forest_wolf.conf")
        .unwrap();
}

fn load_fantasy() -> Container {
    let mut container = Container::new();
    // loader::scenery_space::load(&mut container);
    loader::Loader::load_folders(&mut container, &Path::new("../data/fantasy")).unwrap();
    container
}

#[test]
fn test_fantasy_kill_wolf_and_sell_meat() {
    let mut container = Container::new();
    load_city_forest_wolf_vendor(&mut container);

    let mut scenery = TestScenery::new(container);
    scenery.login();
    scenery_forest_wolf_from_village_to_forest(&mut scenery);
    kill_wolf_and_loot(&mut scenery);
    scenery_forest_wolf_from_forest_to_village(&mut scenery);
    sell_meat(&mut scenery);
}

#[test]
fn test_fantasy_collect_money_should_be_merged() {
    let mut container = Container::new();
    load_city_forest_wolf_vendor(&mut container);

    let mut scenery = TestScenery::new(container);
    scenery.login();

    let chest_id = builder::add_container(&mut scenery.game.container, "chest", ObjId(0), true);
    Loader::spawn_at(&mut scenery.game.container, StaticId(1), chest_id).unwrap();

    pick_money_from_chest(&mut scenery);
    assert_money(&mut scenery, 1);

    Loader::spawn_at(&mut scenery.game.container, StaticId(1), chest_id).unwrap();
    pick_money_from_chest(&mut scenery);
    assert_money(&mut scenery, 2);

    Loader::spawn_at(&mut scenery.game.container, StaticId(1), chest_id).unwrap();
    pick_money_from_chest(&mut scenery);
    assert_money(&mut scenery, 3);
}

#[test]
fn test_fantasy_buy_weapon() {
    let mut container = Container::new();
    load_city_forest_wolf_vendor(&mut container);

    let mut scenery = TestScenery::new(container);
    scenery.login();
    scenery.give_money(1000);
    buy_sword(&mut scenery);
    equip_sword(&mut scenery);
}

#[test]
fn test_fantasy_hire_mercenary_and_fight() {
    let mut scenery = new_scenery(vec![
        "../data/tests/scenery_fantasy_forest_wolf.conf",
        "../data/tests/scenery_fantasy_forest_wolf_mercenary.conf",
    ]);

    scenery.give_money(100);
    // wait until mercenary spawn
    hire_mercenary(&mut scenery);
    scenery_forest_wolf_from_village_to_forest(&mut scenery);
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
    let mut scenery = TestScenery::new(load_fantasy());
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
    let mut scenery = TestScenery::new(load_fantasy());
    scenery.login();
    from_village_to_dungeons(&mut scenery);
    scenery.input("map");
    scenery.assert_output(vec!["Map", "02==**=="]);
}

#[test]
fn test_fantasy_player_respawn() {
    // let mut scenery = TestScenery::new();
    // scenery.login();
    // from_village_to_temple(&mut scenery);
    //
    // scenery
    //     .game
    //     .container
    //     .admin_kill_avatar_from_connection(scenery.connection_id)
    //     .unwrap();
    // scenery.assert_output(vec!["resurrect"]);
    // TODO: create method to kill a avatar
}

fn new_scenery(files: Vec<&str>) -> TestScenery {
    let mut container = Container::new();
    loader::Loader::load_hocon_files(&mut container, &files).unwrap();

    let mut scenery = TestScenery::new(container);
    scenery.login();
    scenery
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
    scenery.input("examine wolf corpse");
    scenery.wait_for("- meat");
    // collect loot
    scenery.input("get meat in wolf corpse");
    scenery.wait_for("you pick");
    scenery.input("inv");
    scenery.wait_for("- meat");
}

fn from_village_to_market(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("Village");
    scenery.input("s");
    scenery.wait_for("For");
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

fn scenery_forest_wolf_from_village_to_forest(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("village");
    scenery.input("s");
    scenery.wait_for("forest");
}

fn scenery_forest_wolf_from_forest_to_village(scenery: &mut TestScenery) {
    scenery.input("look");
    scenery.wait_for("forest");
    scenery.input("n");
    scenery.wait_for("village");
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

fn buy_sword(scenery: &mut TestScenery) {
    scenery.input_and_wait("buy", "sword");
    scenery.input_and_wait("buy sword", "bought");
}

fn equip_sword(scenery: &mut TestScenery) {
    scenery.input_and_wait("equip sword", "you equip");
}

fn assert_money(scenery: &mut TestScenery, expected: u32) {
    scenery.input("inv");
    if expected == 1 {
        scenery.wait_for("gold");
    } else {
        scenery.wait_for(&format!("gold x{}", expected));
    }
}
