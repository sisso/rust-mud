use std::collections::HashMap;
use commons::ObjId;
use logs::*;

pub type CraftId = ObjId;

#[derive(Clone,Debug)]
pub struct Craft {
    pub id: ObjId,
}

impl Craft {
    pub fn new(id: ObjId) -> Self {
        Craft {
            id
        }
    }
}

#[derive(Clone,Debug)]
pub struct Crafts {
    index: HashMap<ObjId, Craft>,
}

impl Crafts {
    pub fn new() -> Self {
        Crafts {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, craft: Craft) {
        assert!(!self.index.contains_key(&craft.id));
        info!("{:?} add {:?}", craft.id, craft);
        self.index.insert(craft.id, craft);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Craft> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&Craft,()> {
        self.index.get(&id).ok_or(())
    }

    pub fn exists(&self, id: ObjId) -> bool { self.index.contains_key(&id) }
}
