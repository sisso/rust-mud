use super::mob::*;
use super::player::*;
use super::room::*;

use crate::utils::*;

#[derive(Clone,Copy,Debug)]
pub struct GameTime {
    pub tick: Tick,
    pub total: Seconds,
    pub delta: Seconds,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dir {
    N,
    S,
    W,
    E
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
}

impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Dir::N => write!(f, "N"),
            Dir::S => write!(f, "S"),
            Dir::E => write!(f, "E"),
            Dir::W => write!(f, "W"),
        }
    }
}

pub struct PlayerCtx<'a> {
    pub player: &'a Player,
    pub avatar: &'a Mob,
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

    pub fn next(&mut self) -> u32 {
        let v = self.next;
        self.next += 1;
        v
    }
}
