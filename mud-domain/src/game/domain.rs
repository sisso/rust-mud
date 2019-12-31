use super::mob::*;
use super::player::*;
use super::room::*;
use crate::errors;
use crate::errors::Error::ParserError;
use commons::{DeltaTime, Tick, TotalTime};
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Modifier(pub i32);

impl Modifier {
    pub fn apply(&self, value: Attribute) -> Attribute{
        0.min(value as i32 + self.0) as u32
    }
}

pub type Attribute = u32;
pub type Rd = u32;

#[derive(Clone, Copy, Debug)]
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
            delta: DeltaTime(0.0),
        }
    }

    pub fn add(&mut self, delta: DeltaTime) {
        self.tick = self.tick.next();
        self.total = self.total + delta;
        self.delta = delta;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Copy, Deserialize)]
pub enum Dir {
    N,
    S,
    W,
    E,
}

impl Dir {
    pub fn inv(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::E,
            Dir::W => Dir::W,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Dir::N => "n",
            Dir::S => "s",
            Dir::E => "e",
            Dir::W => "w",
        }
    }

    pub fn parse(value: &str) -> errors::Result<Dir> {
        match value {
            "n" => Ok(Dir::N),
            "s" => Ok(Dir::S),
            "e" => Ok(Dir::E),
            "w" => Ok(Dir::W),
            _ => Err(ParserError {
                kind: "Dir".to_string(),
                value: value.to_string(),
            }),
        }
    }
}

pub struct MobCtx<'a> {
    pub mob: &'a Mob,
    pub room: &'a Room,
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
        NextId { next: 0 }
    }

    pub fn new_from(value: u32) -> Self {
        NextId { next: value }
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
