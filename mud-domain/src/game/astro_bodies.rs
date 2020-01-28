use commons::ObjId;
use logs::*;
use std::collections::HashMap;

pub type AstroBodyId = ObjId;

/// orbit distance in 1000 * km
pub type DistanceMkm = f32;

#[derive(Clone, Debug, Copy)]
pub enum AstroBodyKind {
    Star,
    Planet,
    Moon,
    Ship,
    AsteroidField,
    Station
}

#[derive(Clone, Debug)]
pub struct AstroBody {
    pub id: AstroBodyId,
    pub orbit_distance: DistanceMkm,
    pub kind: AstroBodyKind,
}

impl AstroBody {
    pub fn new(id: AstroBodyId, orbit_distance: DistanceMkm, kind: AstroBodyKind) -> Self {
        AstroBody { id, orbit_distance, kind }
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
