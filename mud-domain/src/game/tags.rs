use commons::ObjId;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Copy, Eq, PartialEq, Hash)]
pub enum Tag {}

#[derive(Clone, Debug)]
pub struct Tags {
    index: HashMap<ObjId, HashSet<Tag>>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, _obj_id: ObjId, _tag: Tag) {}

    pub fn remove(&mut self, _id: ObjId, _tag_id: ObjId) -> bool {
        false
    }

    pub fn get(&self, id: ObjId) -> HashSet<Tag> {
        self.index.get(&id).cloned().unwrap_or(HashSet::new())
    }
}
