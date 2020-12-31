use crate::errors::{Error, Result};
use crate::game::mob::{MobCommand, MobId};
use crate::game::room::RoomId;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AiCommand {
    Idle,
    Aggressive,
    Passive,
    FollowAndProtect {
        target_id: ObjId,
    },
    Hauler {
        from: ObjId,
        to: ObjId,
        wares: Vec<ObjId>,
    },
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct AiRepo {
    index: HashMap<ObjId, Ai>,
}

impl AiRepo {
    pub fn new() -> Self {
        AiRepo {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, template: Ai) -> Result<()> {
        if self.index.contains_key(&template.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(template.id, template);
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
