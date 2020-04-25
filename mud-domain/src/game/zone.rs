use crate::errors::{Error, Result};
use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// should use live zone
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

#[derive(Clone, Debug)]
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

impl SnapshotSupport for Zones {
    fn save(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, comp) in &self.index {
            let value = json!(comp);
            snapshot.add(id.as_u32(), "zone", value);
        }
    }

    fn load(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}
