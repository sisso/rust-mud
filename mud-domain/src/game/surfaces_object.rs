use commons::ObjId;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceObject {
    pub id: ObjId,
}

impl SurfaceObject {
    pub fn new(id: ObjId) -> Self {
        SurfaceObject { id }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceObjects {
    index: HashMap<ObjId, SurfaceObject>,
}

impl SurfaceObjects {
    pub fn new() -> Self {
        SurfaceObjects {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: SurfaceObject) {
        assert!(!self.index.contains_key(&value.id));
        log::debug!("{:?} added", value.id);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<SurfaceObject> {
        log::debug!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&SurfaceObject, ()> {
        self.index.get(&id).ok_or(())
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}
