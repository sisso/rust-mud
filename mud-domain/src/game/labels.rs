use std::collections::HashMap;
use commons::ObjId;
use crate::utils::text;

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

    pub fn set(&mut self, labels: Label) {
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

    pub fn resolve_codes(&self, ids: &Vec<ObjId>) -> Vec<&str> {
        // flat map can not be used because we want replace none by ???
        ids.iter().map(|id| {
            self.index.get(&id)
                .map(|labels| labels.code.as_str())
                .unwrap_or("???")
        }).collect()
    }

    pub fn resolve_labels(&self, ids: &Vec<ObjId>) -> Vec<&str> {
        ids.iter().map(|id| self.get_label_f(*id)).collect()
    }

    pub fn search_codes(&self, ids: &Vec<ObjId>, input: &str) -> Vec<ObjId> {
        let candidates = self.resolve_labels(&ids);
        let selected = text::search_label(input, &candidates);
        let mut result = vec![];
        for selected_index in selected {
            result.push(ids[selected_index]);
        }
        result
    }
}
