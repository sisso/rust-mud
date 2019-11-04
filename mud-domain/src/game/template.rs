use std::collections::HashMap;
use commons::ObjId;

#[derive(Clone,Debug)]
pub struct Template {
    pub id: ObjId,
}

#[derive(Clone,Debug)]
pub struct Templates {
    index: HashMap<ObjId, Template>,
}

impl Templates {
    pub fn new() -> Self {
        Templates {
            index: HashMap::new(),
        }
    }

    pub fn update(&mut self, template: Template) -> Option<Template> {
        self.index.insert(template.id, template)
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Template> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Template> {
        self.index.get(&id)
    }
}
