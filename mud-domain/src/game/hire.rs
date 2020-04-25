use crate::errors::{Error, Result};
use crate::game::prices::Money;
use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use logs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// should use live hire
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hire {
    pub id: ObjId,
    pub cost: Money,
}

impl Hire {
    pub fn new(id: ObjId) -> Self {
        Hire { id, cost: Money(0) }
    }
}

#[derive(Clone, Debug)]
pub struct Hires {
    index: HashMap<ObjId, Hire>,
}

impl Hires {
    pub fn new() -> Self {
        Hires {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, hire: Hire) -> Result<()> {
        if self.index.contains_key(&hire.id) {
            return Err(Error::ConflictException);
        }
        debug!("{:?} adding hire", hire);
        self.index.insert(hire.id, hire);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Hire> {
        debug!("{:?} remove hire", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Hire> {
        self.index.get(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }
}

impl SnapshotSupport for Hires {
    fn save(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, comp) in &self.index {
            let value = json!(comp);
            snapshot.add(id.as_u32(), "hire", value);
        }
    }

    fn load(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}
