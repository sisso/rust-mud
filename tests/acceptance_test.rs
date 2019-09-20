extern crate mud;

use mud::server;
use mud::server_dummy;
use mud::server_dummy::ServerDummy;
use mud::game::domain::*;
use mud::utils::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use mud::game::server_runner::ServerRunner;
use mud::utils::Second;

struct TestGame {
    outputs: Rc<RefCell<Vec<String>>>,
    inputs: Rc<RefCell<Vec<String>>>,
    game: ServerRunner,
}

const DELTA: Second = Second(1.0);
const SAFE: u32 = 10;
const SAVE: &str = "/tmp/test";

impl TestGame {
    pub fn new() -> Self {
        let server = ServerDummy::new();
        let outputs = server.get_outputs_pointer();
        let inputs = server.get_inputs_pointer();
        let _ = std::fs::remove_file(SAVE);
        let game = ServerRunner::new(Box::new(server), Some((SAVE.to_string(), DELTA)));

        TestGame {
            outputs,
            inputs,
            game,
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
        println!("testserver.get_outputs - {:?}", outputs);
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
