use crate::errors::{Error, Result};
use crate::game::astro_bodies::DistanceMkm;
use crate::game::container::Container;
use commons::{ObjId, TotalTime};
use logs::*;
use serde_json::ser::State;
use std::collections::HashMap;

pub type ShipId = ObjId;

trait TimedState {
    fn is_running(&self, total_time: TotalTime) -> bool {
        self.get_complete_time()
            .map(|time| time.is_after(total_time))
            .unwrap_or(false)
    }

    fn get_complete_time(&self) -> Option<TotalTime>;
}

/// Require a initial state to allow to be set but first step will be managed by the system.
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

impl TimedState for MoveState {
    fn get_complete_time(&self) -> Option<TotalTime> {
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

/// Require a initial state to allow to be set but first step will be managed by the system.
#[derive(Clone, Debug)]
pub enum LaunchState {
    NotStarted,
    Ignition { complete_time: TotalTime },
    Ascending { complete_time: TotalTime },
    Circularization { complete_time: TotalTime },
}

impl TimedState for LaunchState {
    fn get_complete_time(&self) -> Option<TotalTime> {
        match self {
            LaunchState::NotStarted => None,
            LaunchState::Ignition { complete_time, .. } => Some(*complete_time),
            LaunchState::Ascending { complete_time, .. } => Some(*complete_time),
            LaunchState::Circularization { complete_time, .. } => Some(*complete_time),
        }
    }
}

/// Require a initial state to allow to be set but first step will be managed by the system.
#[derive(Clone, Debug)]
pub enum LandState {
    NotStarted,
    Retroburn { complete_time: TotalTime },
    Deorbiting { complete_time: TotalTime },
    AeroBraking { complete_time: TotalTime },
    Approach { complete_time: TotalTime },
    Landing { complete_time: TotalTime },
}

impl TimedState for LandState {
    fn get_complete_time(&self) -> Option<TotalTime> {
        match self {
            LandState::NotStarted => None,
            LandState::Retroburn { complete_time, .. } => Some(*complete_time),
            LandState::Deorbiting { complete_time, .. } => Some(*complete_time),
            LandState::AeroBraking { complete_time, .. } => Some(*complete_time),
            LandState::Approach { complete_time, .. } => Some(*complete_time),
            LandState::Landing { complete_time, .. } => Some(*complete_time),
        }
    }
}

/// Require a initial state to allow to be set but first step will be managed by the system.
#[derive(Clone, Debug)]
pub enum JumpState {
    NotStarted,
    Align { complete_time: TotalTime },
    RechargingCapacitors { complete_time: TotalTime },
    Jumping { complete_time: TotalTime },
}

impl TimedState for JumpState {
    fn get_complete_time(&self) -> Option<TotalTime> {
        match self {
            JumpState::NotStarted => None,
            JumpState::Align { complete_time, .. } => Some(*complete_time),
            JumpState::RechargingCapacitors { complete_time, .. } => Some(*complete_time),
            JumpState::Jumping { complete_time, .. } => Some(*complete_time),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ShipCommand {
    Idle,
    MoveTo {
        target_id: ObjId,
        state: MoveState,
    },
    Launch {
        target_id: ObjId,
        state: LaunchState,
    },
    Land {
        target_id: ObjId,
        state: LandState,
    },
    Jump {
        target_id: ObjId,
        state: JumpState,
    },
}

impl ShipCommand {
    pub fn move_to(target_id: ObjId) -> Self {
        ShipCommand::MoveTo {
            target_id,
            state: MoveState::NotStarted,
        }
    }

    pub fn launch(target_id: ObjId) -> Self {
        ShipCommand::Launch {
            target_id,
            state: LaunchState::NotStarted,
        }
    }

    pub fn land(target_id: ObjId) -> Self {
        ShipCommand::Land {
            target_id,
            state: LandState::NotStarted,
        }
    }

    pub fn jump(target_id: ObjId) -> Self {
        ShipCommand::Jump {
            target_id,
            state: JumpState::NotStarted,
        }
    }

    pub fn is_running(&self, total_time: TotalTime) -> bool {
        match self {
            ShipCommand::Idle => false,
            ShipCommand::MoveTo { state, .. } => state.is_running(total_time),
            ShipCommand::Land { state, .. } => state.is_running(total_time),
            ShipCommand::Launch { state, .. } => state.is_running(total_time),
            ShipCommand::Jump { state, .. } => state.is_running(total_time),
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
        if let Some(ship) = self.index.get_mut(&craft_id) {
            info!("{:?} set command to {:?}", craft_id, command);
            ship.command = command;
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
