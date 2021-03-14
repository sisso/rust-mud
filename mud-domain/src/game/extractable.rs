use crate::errors::{Error, Result};
use crate::game::loader::dto::StaticId;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Extractable {
    pub id: ObjId,
    pub prefab_id: StaticId,
}

#[derive(Clone, Debug)]
pub struct Extractables {
    index: HashMap<ObjId, Extractable>,
}

// TODO: move mostly of this methods to a trait
impl Extractables {
    pub fn new() -> Self {
        Extractables {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: Extractable) -> Result<()> {
        if self.index.contains_key(&e.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(e.id, e);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Extractable> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Extractable> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Extractable> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Extractable> + 'a {
        self.index.values()
    }
}
