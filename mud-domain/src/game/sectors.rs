use std::collections::HashMap;
use commons::ObjId;

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
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: SectorId) -> Option<Sector> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: SectorId) -> Result<&Sector,()> {
        self.index.get(&id).ok_or(())
    }
}
