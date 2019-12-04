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
        loader::Loader::load_folder(&mut container, &Path::new("../data/space"));
        TestScenery {
            game: Game::new(container),
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
            .get_outputs()
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
        for _ in 0..100 {
            let outputs = self.take_outputs();
            let found = outputs.iter().find(|msg| msg.contains(contains));

            if found.is_some() {
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
}

#[test]
fn test_sectormap() -> Result<(), ()> {
    let mut scenery = TestScenery::new();
    scenery.login();
    scenery.send_input("sm");

    // look star map
    let outputs = scenery.wait_for(".@.");
    assert!(outputs.join("\n").contains(".@."));
    //    assert_eq!("", outputs.join("\n").as_str());

    // check move targets
    scenery.send_input("move");
    scenery.wait_for("Dune");

    scenery.send_input("move dune");
    scenery.wait_for("command accepted");
    scenery.wait_for("complete");

    scenery.send_input("land");
    scenery.wait_for("Palace");

    scenery.send_input("land palace");
    scenery.wait_for("complete");

    scenery.send_input("s");
    scenery.send_input("s");
    scenery.send_input("out");
    scenery.wait_for("Light Transport");

    scenery.send_input("enter");
    scenery.send_input("n");
    scenery.send_input("n");
    scenery.send_input("launch");
    scenery.wait_for("launch complete");

    //    scenery.wait_for("Palace");

    //    let outputs = scenery.take_outputs();
    //    assert_eq!("???", outputs.join("\n"));

    Ok(())
}

//#[test]
//fn test_fly_to() -> Result<(),()> {
//    let mut scenery = TestScenery::new();
//    scenery.login();
//    scenery.send_input("sm");
//    let outputs = scenery.take_outputs();
//    assert!(outputs.join("\n").contains(".@."));
//    Ok(())
//}

//use std::cell::RefCell;
//use std::rc::Rc;

// FIXME: uncommment
//struct TestGame {
//    outputs: Rc<RefCell<Vec<String>>>,
//    inputs: Rc<RefCell<Vec<String>>>,
//    game: ServerRunner,
//}
//
//const DELTA: DeltaTime = DeltaTime(1.0);
//const SAFE: u32 = 10;
//const SAVE: &str = "/tmp/test";
//
//impl TestGame {
//    pub fn new() -> Self {
//        let server = ServerDummy::new();
//        let outputs = server.get_outputs_pointer();
//        let inputs = server.get_inputs_pointer();
//        let _ = std::fs::remove_file(SAVE);
//        let game = ServerRunner::new(Box::new(server), Some((SAVE.to_string(), DELTA)));
//
//        TestGame {
//            outputs,
//            inputs,
//            game,
//        }
//    }
//
//    pub fn look_and_wait_for(&mut self, expected: &str) {
//        for _ in 0..SAFE {
//            self.input("look");
//            self.run_tick();
//            if self.get_outputs().iter().find(|i| i.contains(expected)).is_some() {
//                break;
//            }
//        }
//    }
//
//    pub fn wait_for(&mut self, expected: &str) {
//        for _ in 0..SAFE {
//            self.run_tick();
//
//            if self.get_outputs().iter().find(|i| i.contains(expected)).is_some() {
//                return;
//            }
//        }
//
//        panic!(format!("failed to wait for expected '{}' after {} ticks", expected, SAFE));
//    }
//
//    pub fn run_tick(&mut self) {
//        self.game.run(DELTA);
//    }
//
//    pub fn get_outputs(&self) -> Vec<String> {
//        let outputs = self.outputs.replace(vec![]);
//        println!("testserver.get_outputs - {:?}", outputs);
//        outputs
//    }
//
//    pub fn input(&mut self, input: &str) {
//        self.inputs.borrow_mut().push(input.to_string());
//    }
//}
//
//#[test]
//fn kill_something() {
//    let mut g = TestGame::new();
//    do_login(&mut g);
//    do_move_to_bar_wait_for_drunk(&mut g);
//    g.input("kill Drunk");
//    g.wait_for("killed");
//    g.input("examine body");
//    g.run_tick();
//    let _ = g.get_outputs();
//    g.input("pick body coins");
//    g.run_tick();
//    let _ = g.get_outputs();
//    g.input("stats");
//    g.run_tick();
//    assert!(g.get_outputs().iter().find(|msg| msg.contains("- coins (2)")).is_some());
//}
//
//#[test]
//fn admin_kill_test() {
//    let mut g = TestGame::new();
//    do_login(&mut g);
//    g.input("admin suicide");
//    g.run_tick();
//    g.wait_for("you have resurrected!");
//    do_look_main_room(&mut g);
//    g.input("rest");
//    g.wait_for("sit and rest");
//    g.wait_for("healing");
//    g.wait_for("healed");
//    g.input("stand");
//    do_move_to_bar_wait_for_drunk(&mut g);
//}
//
//#[test]
//fn pickup_equipment_at_florest() {
//    let mut g = TestGame::new();
//    do_login(&mut g);
//    do_move_to_florest(&mut g);
//    g.look_and_wait_for("sword");
//    g.look_and_wait_for("armor");
//    g.input("get sword");
//    g.wait_for("you pick a sword");
//    g.input("get armor");
//    g.wait_for("you pick a armor");
//    g.input("stats");
//    g.run_tick();
//
//    let outputs = g.get_outputs();
//    assert!(outputs.iter().find(|msg| msg.contains("- armor")).is_some());
//    assert!(outputs.iter().find(|msg| msg.contains("- sword")).is_some());
//
//    g.input("equip sword");
//    g.run_tick();
//
//    g.input("equip armor");
//    g.run_tick();
//
//    g.input("stats");
//    g.run_tick();
//
//    let outputs = g.get_outputs();
//    assert!(outputs.iter().find(|msg| msg.contains("- armor*")).is_some());
//    assert!(outputs.iter().find(|msg| msg.contains("- sword*")).is_some());
//}
//
//fn do_move_to_bar_wait_for_drunk(g: &mut TestGame) {
//    g.input("s");
//    g.wait_for("Bar");
//    g.look_and_wait_for("Drunk");
//}
//
//fn do_move_to_florest(g: &mut TestGame) {
//    g.input("s");
//    g.wait_for("Bar");
//    g.input("s");
//    g.wait_for("Florest");
//}
//
//fn do_login(g: &mut TestGame) {
//    g.wait_for("Welcome to MUD");
//    g.input("sisso");
//    g.wait_for("welcome sisso");
//    do_look_main_room(g);
//}
//
//fn do_look_main_room(g: &mut TestGame) {
//    g.input("look");
//    g.wait_for("Main Room");
//}
