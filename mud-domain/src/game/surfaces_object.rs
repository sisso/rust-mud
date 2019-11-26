use commons::ObjId;
use logs::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SurfaceObject {
    pub id: ObjId,
}

impl SurfaceObject {
    pub fn new(id: ObjId) -> Self {
        SurfaceObject { id }
    }
}

#[derive(Clone, Debug)]
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
        debug!("{:?} added", value.id);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<SurfaceObject> {
        debug!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&SurfaceObject, ()> {
        self.index.get(&id).ok_or(())
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}
