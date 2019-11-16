extern crate mud_domain;
extern crate logs;

use commons::{DeltaTime, PlayerId};
use mud_domain::game::{spawn, mob, item, OutputsBuffer, view_main, Output, find_players_per_room, loader};
use mud_domain::game::container::{Container, Ctx};


use std::collections::HashMap;


pub struct Engine {
    container: Container,
    outputs: OutputsBuffer,
}

impl Engine {
    pub fn new() -> Self {
        let container = Container::new();

        Engine {
            container,
            outputs: OutputsBuffer::new(),
        }
    }

    pub fn load(&mut self, _data_dir: &str) {
        loader::load(&mut self.container);
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        self.container.tick(&mut self.outputs, delta_time);

        let mut ctx = Ctx {
            container: &mut self.container,
            outputs: &mut self.outputs,
        };

        spawn::run(&mut ctx);
        mob::run_tick(&mut ctx);
        item::run_tick(&mut ctx);
    }

    pub fn disconnect(&mut self, _player_id: PlayerId) {
        unimplemented!()
    }

    pub fn take_events(&mut self) -> Vec<ConnectionEvent> {
        let mut result: HashMap<PlayerId, Vec<Event>> = HashMap::new();
        let mut players_per_room = find_players_per_room(&self.container);

        // for each output convert into personal events to players
        for output in self.outputs.take() {
            match output {
                Output::Private { player_id, msg } => {
                    let event = Event::Generic { msg };
                    result.entry(player_id).or_default().push(event);
                },
                Output::Room { player_id, room_id, msg } => {
                    let players =
                        players_per_room.entry(room_id).or_default().iter()
                            .filter(|candidate_player_id| {
                                match player_id {
                                    Some(player_id) if **candidate_player_id == player_id => false,
                                    _ => true
                                }
                            });

                    for player_id in players {
                        let event = Event::Generic { msg: msg.clone() };
                        result.entry(*player_id).or_default().push(event);
                    }
                }
            }
        }

        result.into_iter().map(|(player_id, events)| ConnectionEvent { player_id, events } ).collect()
    }

    pub fn login(&mut self, _login: &str, _pass: &str) -> Result<PlayerId, ()> {
//        let player_id = add_player(&mut self.container, login);
//        Ok(player_id)
        // TODO: copy from game
        unimplemented!()
    }

    // TODO: move to process events
    pub fn add_action(&mut self, player_id: PlayerId, action: Action) {
        match action {
            Action::Generic { input } => {
                view_main::handle(&mut self.container, &mut self.outputs, player_id, input.as_str());
            },
            _ => panic!()
        }
    }
}

#[derive(Debug,Clone)]
pub enum Action {
    Move,
    Get,
    Attack,
    Rest,
    Stand,
    Stats,
    Look,
    Examine,
    Equip,
    Say,
    Uptime,
    Score,
    Generic { input: String },
}

#[derive(Debug,Clone)]
pub struct ConnectionEvent {
    pub player_id: PlayerId,
    pub events: Vec<Event>,
}

#[derive(Debug,Clone)]
pub enum Event {
    LookRoom,
    LookObject,
    LookFail,
    Say,
    OtherSay,
    Move,
    OtherMoveIn,
    OtherMoveOut,
    FailToMove,
    AttackMiss,
    AttackHit,
    AttackedMiss,
    AttackedHit,
    OtherAttackMiss,
    OtherAttackHit,
    AttackFail,
    Killed,
    OtherKilled,
    Stats,
    Inventory,
    GetFail,
    Get,
    OtherGet,
    EquipFail,
    Equip,
    Rest,
    RestStop,
    OtherRest,
    OtherRestStop,
    Generic { msg: String },
}
