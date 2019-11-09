use std::collections::{HashMap, HashSet};
use commons::ObjId;

#[derive(Clone,Debug,Copy, Eq, PartialEq, Hash)]
pub enum Tag {
}

#[derive(Clone,Debug)]
pub struct Tags {
    index: HashMap<ObjId, HashSet<Tag>>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, obj_id: ObjId, tag: Tag) {
    }

    pub fn remove(&mut self, id: ObjId, tag_id: ObjId) -> bool {
        false
    }

    pub fn get(&self, id: ObjId) -> HashSet<Tag> {
        self.index.get(&id).cloned().unwrap_or(HashSet::new())
    }
}
