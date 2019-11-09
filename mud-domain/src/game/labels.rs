use std::collections::HashMap;
use commons::ObjId;

#[derive(Clone,Debug)]
pub struct Label {
    pub id: ObjId,
    /// how we call it
    pub label: String,
    /// tokens used to reference in commends
    pub code: String,
    pub desc: String,
}

impl Label {
    pub fn new(id: ObjId, label: &str) -> Self {
        Label {
            id,
            label: label.to_string(),
            code: label.to_string(),
            desc: label.to_string(),
        }
    }

    pub fn new_desc(id: ObjId, label: &str, desc: &str) -> Self {
        Label {
            id,
            label: label.to_string(),
            code: label.to_string(),
            desc: desc.to_string(),
        }
    }
}

#[derive(Clone,Debug)]
pub struct Labels {
    index: HashMap<ObjId, Label>,
}

impl Labels {
    pub fn new() -> Self {
        Labels {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, labels: Label) {
        assert!(!self.index.contains_key(&labels.id));
        self.index.insert(labels.id, labels);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Label> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Result<&Label,()> {
        self.index.get(&id).ok_or(())
    }

    pub fn get_label(&self, id: ObjId) -> Result<&str,()> {
        self.index.get(&id)
            .map(|label| label.label.as_str())
            .ok_or(())
    }

    pub fn get_label_f(&self, id: ObjId) -> &str {
        self.index.get(&id)
            .map(|label| label.label.as_str())
            .unwrap_or("???")
    }

}
