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

    pub fn add(&mut self, template: Template) {
        assert!(!self.index.contains_key(&template.id));
        self.index.insert(template.id, template);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Template> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&Template,()> {
        self.index.get(&id).ok_or(())
    }
}
