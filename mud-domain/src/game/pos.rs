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

    pub fn set(&mut self, id: ObjId, value: V2) {
        self.index.insert(id, Pos {
            id,
            pos: value
        });
    }

    pub fn remove(&mut self, id: ObjId) -> Option<V2> {
        info!("{:?} removed", id);
        self.index.remove(&id).map(|i| i.pos)
    }

    pub fn get_pos(&self, id: ObjId) -> Option<V2> {
        self.index.get(&id).map(|i| i.pos)
    }

    pub fn update(&mut self, id: ObjId, new_pos: V2) -> UResult {
        self.index.get_mut(&id).map(|i| {
            info!("{:?} set_pos {:?}", id, new_pos);
            i.pos = new_pos;
            ()
        }).ok_or(())
    }
}
