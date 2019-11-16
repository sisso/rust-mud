use super::mob::*;
use super::player::*;
use super::room::*;
use commons::{DeltaTime, Tick, TotalTime};

#[derive(Clone,Copy,Debug)]
pub struct GameTime {
    pub tick: Tick,
    pub total: TotalTime,
    pub delta: DeltaTime,
}

impl GameTime {
    pub fn new() -> Self {
        GameTime {
            tick: Tick(0),
            total: TotalTime(0.0),
            delta: DeltaTime(0.0)
        }
    }

    pub fn add(&mut self, delta: DeltaTime) {
        self.tick = self.tick.next();
        self.total = self.total + delta;
        self.delta = delta;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum Dir {
    N,
    S,
    W,
    E,
    Enter,
    Out,
}

impl Dir {
    pub fn inv(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::E,
            Dir::W => Dir::W,
            Dir::Enter => Dir::Out,
            Dir::Out => Dir::Enter,
        }
    }

    pub fn as_str(&self) -> &'static str {
       match self {
           Dir::N => "n",
           Dir::S => "s",
           Dir::E => "e",
           Dir::W => "w",
           Dir::Enter => "enter",
           Dir::Out => "out",
       }
    }
}

pub struct PlayerCtx<'a> {
    pub player: &'a Player,
    pub mob: &'a Mob,
    pub room: &'a Room,
}

pub struct NextId {
    next: u32,
}

impl NextId {
    pub fn new() -> Self {
        NextId {
            next: 0
        }
    }

    pub fn new_from(value: u32) -> Self {
        NextId {
            next: value
        }
    }

    pub fn next(&mut self) -> u32 {
        let v = self.next;
        self.next += 1;
        v
    }

    pub fn set_max(&mut self, id: u32) {
        self.next = self.next.max(id + 1);
    }
}
