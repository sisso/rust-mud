use std::collections::HashMap;
use commons::ObjId;
use logs::*;

pub type SurfaceId = ObjId;

#[derive(Clone,Debug)]
pub struct Surface {
    pub id: SurfaceId,
    pub size: u32,
    pub is_3d: bool,
}

impl Surface {
    pub fn new(id: SurfaceId) -> Self {
        Surface { id, size: 10, is_3d: false }
    }
}

#[derive(Clone,Debug)]
pub struct Surfaces {
    index: HashMap<SurfaceId, Surface>,
}

impl Surfaces {
    pub fn new() -> Self {
        Surfaces {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: Surface) {
        assert!(!self.index.contains_key(&value.id));
        info!("{:?} add {:?}", value.id, value);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: SurfaceId) -> Option<Surface> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: SurfaceId) -> Result<&Surface,()> {
        self.index.get(&id).ok_or(())
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}