use std::collections::HashMap;
use crate::game::obj::ObjId;

#[derive(Clone,Debug)]
pub struct Template {
    pub id: ObjId,
}

#[derive(Clone,Debug)]
pub struct Templates {
    templates: HashMap<ObjId, Template>,
}

impl Templates {
    pub fn new() -> Self {
        Templates {
            templates: HashMap::new(),
        }
    }

    pub fn update(&mut self, template: Template) -> Option<Template> {
        self.templates.insert(template.id, template)
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Template> {
        self.templates.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Template> {
        self.templates.get(&id)
    }
}
