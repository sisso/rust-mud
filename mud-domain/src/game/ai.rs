use crate::errors::{Error, Result};
use crate::game::loader::dto::{
    AiCommandAggressivePatrolHomeData, AiData, CanLoad, CanSnapshot, ObjData,
};
use crate::game::mob::{MobCommand, MobId};
use crate::game::room::RoomId;
use commons::ObjId;
use logs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: implement those commands, they are not in use
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiCommand {
    Idle,
    /// attack any enemy that enter in the room
    Aggressive,
    /// ignore others, but return combat if attacked
    Passive,
    /// like aggressive, but keep moving into 'distance' from its spawn point
    AggressivePatrolHome {
        /// distance from spawn, zero means stay in the same room
        distance: u32,
    },
    FollowAndProtect {
        target_id: ObjId,
    },
    Hauler {
        from: ObjId,
        to: ObjId,
    },
    Extract {
        from: ObjId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ai {
    pub id: ObjId,
    pub command: AiCommand,
    /// can have its commands change by owner?
    pub commandable: bool,
}

impl Ai {
    pub fn new(id: ObjId) -> Self {
        Ai {
            id,
            command: AiCommand::Idle,
            commandable: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiRepo {
    index: HashMap<ObjId, Ai>,
}

impl AiRepo {
    pub fn new() -> Self {
        AiRepo {
            index: HashMap::new(),
        }
    }

    pub fn add_or_update(&mut self, ai: Ai) -> Result<()> {
        if self.index.contains_key(&ai.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(ai.id, ai);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Ai> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Ai> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Ai> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Ai> + 'a {
        self.index.values()
    }
}

impl CanLoad for AiRepo {
    fn load(&mut self, obj_id: ObjId, data: &ObjData) -> Result<()> {
        if let Some(ai_data) = &data.ai {
            let ai = parse_ai(obj_id, ai_data);
            self.add_or_update(ai).unwrap();
        }

        Ok(())
    }
}

impl CanSnapshot for AiRepo {
    fn snapshot(&self, obj_id: ObjId, data: &mut ObjData) -> Result<()> {
        match self.index.get(&obj_id) {
            Some(ai) => data.ai = Some(serialize_ai(ai)),
            None => {}
        }

        Ok(())
    }
}

pub fn serialize_ai(ai: &Ai) -> AiData {
    AiData {
        command_aggressive: if ai.command == AiCommand::Aggressive {
            Some(true)
        } else {
            None
        },
        command_follow_and_protect: match ai.command {
            AiCommand::FollowAndProtect { target_id } => Some(target_id),
            _ => None,
        },
        command_haul: match &ai.command {
            AiCommand::Hauler { from, to } => todo!(),
            // AiCommand::Hauler { from, to, wares } => Some(AiCommandHaulData {
            //     from_id: *from,
            //     to_id: *to,
            //     targets: wares.clone(),
            // }),
            _ => None,
        },
        command_aggressive_patrol_home: match &ai.command {
            AiCommand::AggressivePatrolHome { distance } => {
                Some(AiCommandAggressivePatrolHomeData {
                    distance: *distance,
                })
            }
            _ => None,
        },
        commandable: if ai.commandable { Some(true) } else { None },
    }
}

pub fn parse_ai(obj_id: ObjId, ai_data: &AiData) -> Ai {
    let command = if ai_data.command_aggressive.unwrap_or(false) {
        AiCommand::Aggressive
    } else if let Some(target_id) = ai_data.command_follow_and_protect {
        AiCommand::FollowAndProtect { target_id }
    } else if let Some(haul) = &ai_data.command_haul {
        // AiCommand::Hauler {
        //     from: haul.from_id.clone(),
        //     to: haul.to_id.clone(),
        //     wares: haul.targets.clone(),
        // }
        panic!("not implemented")
    } else if let Some(patrol_data) = &ai_data.command_aggressive_patrol_home {
        AiCommand::AggressivePatrolHome {
            distance: patrol_data.distance,
        }
    } else {
        AiCommand::Idle
    };

    Ai {
        id: obj_id,
        command: command,
        commandable: ai_data.commandable.unwrap_or(false),
    }
}
