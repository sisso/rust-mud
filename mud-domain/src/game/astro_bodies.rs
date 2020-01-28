use commons::ObjId;
use logs::*;
use std::collections::HashMap;

pub type AstroBodyId = ObjId;

/// orbit distance in 1000 * km
pub type DistanceMkm = f32;

#[derive(Clone, Debug)]
pub struct AstroBodyOrbit {
    /// ID of body this astro is orbiting
    pub parent_id: AstroBodyId,
    pub distance: DistanceMkm,
}

#[derive(Clone, Debug)]
pub struct AstroBody {
    pub id: AstroBodyId,
    pub orbit: Option<AstroBodyOrbit>,
}

impl AstroBody {
    pub fn new(id: AstroBodyId) -> Self {
        AstroBody { id, orbit: None }
    }
}

#[derive(Clone, Debug)]
pub struct AstroBodies {
    index: HashMap<AstroBodyId, AstroBody>,
}

impl AstroBodies {
    pub fn new() -> Self {
        AstroBodies {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: AstroBody) {
        assert!(!self.index.contains_key(&value.id));
        info!("{:?} add {:?}", value.id, value);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: AstroBodyId) -> Option<AstroBody> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: AstroBodyId) -> Option<&AstroBody> {
        self.index.get(&id)
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}
