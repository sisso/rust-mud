use logs::*;
use std::collections::HashMap;
use crate::game::container::Container;
use commons::ObjId;
use crate::errors::{Result, Error};

// TODO: rename to ShipId
pub type CraftId = ObjId;

#[derive(Clone, Debug)]
pub enum CraftCommand {
    Idle,
    MoveTo { target_id: ObjId },
}

#[derive(Clone, Debug)]
pub struct CraftAttributes {
    pub speed: f32,
}

impl CraftAttributes {
    pub fn new() -> Self {
        CraftAttributes { speed: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Craft {
    pub id: ObjId,
    pub command: CraftCommand,
    pub attributes: CraftAttributes,
}

impl Craft {
    pub fn new(id: ObjId) -> Self {
        Craft {
            id,
            command: CraftCommand::Idle,
            attributes: CraftAttributes::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Crafts {
    index: HashMap<ObjId, Craft>,
}

impl Crafts {
    pub fn new() -> Self {
        Crafts {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, craft: Craft) {
        assert!(!self.index.contains_key(&craft.id));
        info!("{:?} add {:?}", craft.id, craft);
        self.index.insert(craft.id, craft);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Craft> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Craft> {
        self.index.get(&id)
    }

    pub fn set_command(&mut self, craft_id: CraftId, command: CraftCommand) -> Result<()> {
        if let Some(craft) = self.index.get_mut(&craft_id) {
            info!("{:?} set command to {:?}", craft_id, command);
            craft.command = command;
            Ok(())
        } else {
            Err(Error::IllegalArgument)
        }
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list(&self) -> Vec<CraftId> {
        self.index.keys().cloned().collect()
    }

    pub fn list_all<'a>(&'a self) -> impl Iterator<Item=&'a Craft> + 'a {
        self.index.values()
    }
}
