use std::collections::HashMap;
use commons::ObjId;

pub(crate) type PlanetId = ObjId;

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
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: PlanetId) -> Option<Planet> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: PlanetId) -> Result<&Planet,()> {
        self.index.get(&id).ok_or(())
    }
}

