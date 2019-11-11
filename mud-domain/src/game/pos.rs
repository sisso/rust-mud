use std::collections::HashMap;
use commons::{ObjId, V2};

#[derive(Clone,Debug)]
pub struct Pos {
    pub id: ObjId,
    pub pos: V2,
}

#[derive(Clone,Debug)]
pub struct PosRepo {
    index: HashMap<ObjId, Pos>,
}

impl PosRepo {
    pub fn new() -> Self {
        PosRepo {
            index: HashMap::new(),
        }
    }

    pub fn set(&mut self, value: Pos) {
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Pos> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&Pos,()> {
        self.index.get(&id).ok_or(())
    }
}
