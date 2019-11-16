use std::collections::HashMap;
use commons::{ObjId, V2, UResult};
use logs::*;

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
        info!("{:?} added", value);
        self.index.insert(value.id, value);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Pos> {
        info!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&Pos,()> {
        self.index.get(&id).ok_or(())
    }

    pub fn get_pos(&self, id: ObjId) -> Result<V2, ()> {
        self.index.get(&id).map(|i| i.pos).ok_or(())
    }

    pub fn set_pos(&mut self, id: ObjId, new_pos: V2) -> UResult {
        self.index.get_mut(&id).ok_or(()).map(|i| {
            info!("{:?} set_pos {:?}", id, new_pos);
            i.pos = new_pos;
            ()
        })
    }
}
