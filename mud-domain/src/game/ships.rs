use crate::errors::{Error, Result};
use crate::game::astro_bodies::DistanceMkm;
use crate::game::container::Container;
use commons::{ObjId, TotalTime};
use logs::*;
use serde_json::ser::State;
use std::collections::HashMap;

pub type ShipId = ObjId;

#[derive(Clone, Debug)]
pub enum MoveState {
    NotStarted,
    Alignment {
        complete_time: TotalTime,
    },
    EjectionBurn {
        complete_time: TotalTime,
    },
    Drift {
        from_distance: DistanceMkm,
        to_distance: DistanceMkm,
        start_time: TotalTime,
        complete_time: TotalTime,
    },
    RetroBurn {
        complete_time: TotalTime,
    },
    OrbitSync {
        complete_time: TotalTime,
    },
}

impl MoveState {
    pub fn is_running(&self, total_time: TotalTime) -> bool {
        self.get_complete_time()
            .map(|time| time.is_after(total_time))
            .unwrap_or(false)
    }

    pub fn get_complete_time(&self) -> Option<TotalTime> {
        match self {
            MoveState::NotStarted => None,
            MoveState::Alignment { complete_time, .. } => Some(*complete_time),
            MoveState::EjectionBurn { complete_time, .. } => Some(*complete_time),
            MoveState::Drift { complete_time, .. } => Some(*complete_time),
            MoveState::RetroBurn { complete_time, .. } => Some(*complete_time),
            MoveState::OrbitSync { complete_time, .. } => Some(*complete_time),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ShipCommand {
    Idle,
    MoveTo { target_id: ObjId, state: MoveState },
}

impl ShipCommand {
    pub fn move_to(target_id: ObjId) -> Self {
        ShipCommand::MoveTo {
            target_id,
            state: MoveState::NotStarted,
        }
    }

    pub fn is_running(&self, total_time: TotalTime) -> bool {
        match self {
            ShipCommand::Idle => false,
            ShipCommand::MoveTo { state, .. } => state.is_running(total_time),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShipAttributes {
    pub speed: f32,
}

impl ShipAttributes {
    pub fn new() -> Self {
        ShipAttributes { speed: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Ship {
    pub id: ObjId,
    pub command: ShipCommand,
    pub attributes: ShipAttributes,
}

impl Ship {
    pub fn new(id: ObjId) -> Self {
        Ship {
            id,
            command: ShipCommand::Idle,
            attributes: ShipAttributes::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ships {
    index: HashMap<ObjId, Ship>,
}

impl Ships {
    pub fn new() -> Self {
        Ships {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, ship: Ship) {
        assert!(!self.index.contains_key(&ship.id));
        info!("{:?} add {:?}", ship.id, ship);
        self.index.insert(ship.id, ship);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Ship> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Ship> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Ship> {
        self.index.get_mut(&id)
    }

    pub fn set_command(&mut self, craft_id: ShipId, command: ShipCommand) -> Result<()> {
        if let Some(craft) = self.index.get_mut(&craft_id) {
            info!("{:?} set command to {:?}", craft_id, command);
            craft.command = command;
            Ok(())
        } else {
            Err(Error::InvalidArgumentFailure)
        }
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list(&self) -> Vec<ShipId> {
        self.index.keys().cloned().collect()
    }

    pub fn list_all<'a>(&'a self) -> impl Iterator<Item = &'a Ship> + 'a {
        self.index.values()
    }

    pub fn list_all_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Ship> + 'a {
        self.index.values_mut()
    }
}
