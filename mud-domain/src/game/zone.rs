use crate::errors::{Error, Result};
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hierarchic organization of zones can contains another zones and rooms
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    pub id: ObjId,
}

impl Zone {
    pub fn new(id: ObjId) -> Self {
        Zone { id }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zones {
    index: HashMap<ObjId, Zone>,
}

impl Zones {
    pub fn new() -> Self {
        Zones {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, zone: Zone) -> Result<()> {
        if self.index.contains_key(&zone.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(zone.id, zone);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Zone> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Zone> {
        self.index.get(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }
}
