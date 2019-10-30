extern crate mud_domain;
extern crate logs;

use commons::{DeltaTime, PlayerId, ConnectionId, TotalTime, Tick, Second};
use mud_domain::game::{Game, Ctx, spawn, mob, item, OutputsImpl, view_main, Output, find_players_per_room};
use mud_domain::game::container::Container;
use mud_domain::game::player::add_player;
use mud_domain::game::domain::GameTime;
use std::collections::HashMap;
use logs::*;

pub struct Engine {
    time: GameTime,
    container: Container,
    outputs: OutputsImpl,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            time: GameTime {
                tick: Tick::new(),
                total: Second(0.0),
                delta: Second(0.0),
            },
            container: Container::new(),
            outputs: OutputsImpl::new(),
        }
    }

    pub fn load(&mut self, data_dir: &str) {
        // TODO: implement
        mud_domain::game::loader::load(&mut self.container);
    }

    pub fn tick(&mut self, delta_time: DeltaTime) {
        self.time.tick = self.time.tick.next();
        self.time.total = Second(self.time.total.0 + delta_time.0);
        self.time.delta = delta_time.as_second();

        let mut ctx = Ctx {
            time: &self.time,
            container: &mut self.container,
            outputs: &mut self.outputs,
        };

        spawn::run(&mut ctx);
        mob::run_tick(&mut ctx);
        item::run_tick(&mut ctx);
    }

    pub fn disconnect(&mut self, player_id: PlayerId) {
        self.container.players.player_disconnect(player_id);
    }

    pub fn take_events(&mut self) -> Vec<ConnectionEvent> {
        let mut result: HashMap<PlayerId, Vec<Event>> = HashMap::new();
        let mut players_per_room = find_players_per_room(&self.container);

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

    pub fn login(&mut self, login: &str, pass: &str) -> Result<PlayerId, ()> {
        let player_id = add_player(&mut self.container, login);
        Ok(player_id)
    }

    pub fn add_action(&mut self, player_id: PlayerId, action: Action) {
        match action {
            Action::Other { input } => {
                debug!("{:?} handling input '{}'", player_id, input);
                view_main::handle(&self.time, &mut self.container, &mut self.outputs, player_id, input.as_str());
            },
            _ => panic!()
        }
    }
}

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
    Other { input: String },
}

pub struct ConnectionEvent {
    player_id: PlayerId,
    events: Vec<Event>,
}

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
