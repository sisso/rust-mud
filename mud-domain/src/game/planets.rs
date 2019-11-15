use std::collections::HashMap;
use commons::ObjId;
use logs::*;

pub type PlanetId = ObjId;

#[derive(Clone,Debug)]
pub struct Planet {
    pub id: PlanetId,
}

impl Planet {
    pub fn new(id: PlanetId) -> Self {
        Planet { id }
    }
}

#[derive(Clone,Debug)]
pub struct Planets {
    index: HashMap<PlanetId, Planet>,
}

impl Planets {
    pub fn new() -> Self {
        Planets {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: Planet) {
        assert!(!self.index.contains_key(&value.id));
        info!("{:?} add {:?}", value.id, value);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: PlanetId) -> Option<Planet> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: PlanetId) -> Result<&Planet,()> {
        self.index.get(&id).ok_or(())
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}

