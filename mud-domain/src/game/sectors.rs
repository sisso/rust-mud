use std::collections::HashMap;
use commons::ObjId;
use logs::*;

pub(crate) type SectorId = ObjId;

#[derive(Clone,Debug)]
pub struct Sector {
    pub id: SectorId,
}

impl Sector {
    pub fn new(id: SectorId) -> Self {
        Sector { id }
    }
}

#[derive(Clone,Debug)]
pub struct Sectors {
    index: HashMap<SectorId, Sector>,
}

impl Sectors {
    pub fn new() -> Self {
        Sectors {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: Sector) {
        assert!(!self.index.contains_key(&value.id));
        info!("{:?} add {:?}", value.id, value);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: SectorId) -> Option<Sector> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: SectorId) -> Result<&Sector,()> {
        self.index.get(&id).ok_or(())
    }

    pub fn exists(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}
