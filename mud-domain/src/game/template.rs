use crate::errors::{Error, Result};
use commons::ObjId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Template {
    pub id: ObjId,
}

impl Template {
    pub fn new(id: ObjId) -> Self {
        Template { id }
    }
}

#[derive(Clone, Debug)]
pub struct Templates {
    index: HashMap<ObjId, Template>,
}

// TODO: move mostly of this methods to a trait
impl Templates {
    pub fn new() -> Self {
        Templates {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, template: Template) -> Result<()> {
        if self.index.contains_key(&template.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(template.id, template);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Template> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Template> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Template> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Template> + 'a {
        self.index.values()
    }
}
