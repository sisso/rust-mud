use commons::ObjId;
use logs::*;
use std::collections::HashMap;

pub type AstroBodyId = ObjId;

#[derive(Clone, Debug)]
pub struct AstroBody {
    pub id: AstroBodyId,
    pub orbit_id: Option<AstroBodyId>,
}

impl AstroBody {
    pub fn new(id: AstroBodyId) -> Self {
        AstroBody { id, orbit_id: None }
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
