use crate::errors::Result;
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    id: ObjId,
    know_ids: HashSet<ObjId>,
}

impl Memory {
    pub fn new(id: ObjId) -> Self {
        Memory {
            id,
            know_ids: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Memories {
    index: HashMap<ObjId, Memory>,
}

impl Memories {
    pub fn new() -> Self {
        Memories {
            index: Default::default(),
        }
    }

    pub fn is_know(&self, obj_id: ObjId, other_id: ObjId) -> bool {
        self.index
            .get(&obj_id)
            .map(|memory| memory.know_ids.contains(&other_id))
            .unwrap_or(false)
    }

    pub fn add(&mut self, obj_id: ObjId, other_id: ObjId) -> Result<()> {
        let memory = self.index.entry(obj_id).or_insert(Memory::new(obj_id));
        memory.know_ids.insert(other_id);
        Ok(())
    }
}
