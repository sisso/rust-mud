use crate::utils::strinput::StrInput;
use crate::utils::text;
use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use logs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::panic::resume_unwind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub id: ObjId,
    /// how we call it
    pub label: String,
    /// tokens used to reference in commends
    // TODO: array
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

#[derive(Clone, Debug)]
pub struct Labels {
    index: HashMap<ObjId, Label>,
}

impl Labels {
    pub fn new() -> Self {
        Labels {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, label: Label) {
        assert!(!self.index.contains_key(&label.id));
        debug!("{:?} added", label);
        self.index.insert(label.id, label);
    }

    pub fn update(&mut self, label: Label) {
        assert!(self.index.contains_key(&label.id));
        debug!("{:?} updated", label);
        self.index.insert(label.id, label);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Label> {
        debug!("{:?} removed", id);
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Label> {
        self.index.get(&id)
    }

    pub fn get_label(&self, id: ObjId) -> Option<&str> {
        self.index.get(&id).map(|label| label.label.as_str())
    }

    pub fn get_label_f(&self, id: ObjId) -> &str {
        self.index
            .get(&id)
            .map(|label| label.label.as_str())
            .unwrap_or("???")
    }

    pub fn resolve(&self, ids: &Vec<ObjId>) -> Vec<&Label> {
        ids.iter()
            .map(|&id| self.get(id).expect("label not found"))
            .collect()
    }

    // TODO: do not replace codes per ???
    pub fn resolve_codes(&self, ids: &Vec<ObjId>) -> Vec<&str> {
        // flat map can not be used because we want replace none by ???
        ids.iter()
            .map(|id| {
                self.index
                    .get(&id)
                    .map(|labels| labels.code.as_str())
                    .unwrap_or("???")
            })
            .collect()
    }

    pub fn resolve_labels(&self, ids: &Vec<ObjId>) -> Vec<&str> {
        ids.iter().map(|id| self.get_label_f(*id)).collect()
    }

    pub fn search_codes(&self, ids: &Vec<ObjId>, input: &str) -> Vec<ObjId> {
        if input.is_empty() {
            return vec![];
        }

        let candidates = self.resolve_codes(&ids);
        let selected = text::search_label(input, &candidates);
        let mut result = vec![];
        for selected_index in selected {
            result.push(ids[selected_index]);
        }
        result
    }

    pub fn search(&self, ids: &Vec<ObjId>, input: &str) -> Vec<ObjId> {
        let labels = self.resolve(ids);
        label_search(&labels, input)
    }

    // TODO: this signature is horrible, we need a better way to store prefab ObjData.codes
    pub fn get_code(label: &str, codes: &Option<Vec<String>>) {}
}

// TODO: search by multiple strings (drop sword shield bag)
pub fn label_search<'a>(labels: &Vec<&'a Label>, input: &str) -> Vec<ObjId> {
    let mut result = vec![];
    // search by exactly label
    for i in labels.iter() {
        if text::is_text_eq(i.label.as_str(), input) {
            result.push(i.id);
        }
    }

    if !result.is_empty() {
        return result;
    }

    // search by exactly code
    for i in labels.iter() {
        if text::is_text_eq(i.code.as_str(), input) {
            result.push(i.id);
        }
    }

    if !result.is_empty() {
        return result;
    }

    // search by fuzzy label
    for i in labels.iter() {
        if text::is_text_like(i.label.as_str(), input) {
            result.push(i.id);
        }
    }

    if !result.is_empty() {
        return result;
    }

    // search by fuzzy code
    for i in labels.iter() {
        if text::is_text_like(i.code.as_str(), input) {
            result.push(i.id);
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_label_search(labels: Vec<(u32, &str, &str)>, input: &str, expected: Vec<u32>) {
        let labels: Vec<Label> = labels
            .into_iter()
            .map(|(id, label, code)| Label {
                id: ObjId(id),
                label: label.to_string(),
                code: code.to_string(),
                desc: "".to_string(),
            })
            .collect();

        let labels_ref: Vec<&Label> = labels.iter().map(|i| i).collect();
        let result = label_search(&labels_ref, input);
        let result_ids: Vec<u32> = result.into_iter().map(|id| id.as_u32()).collect();
        assert_eq!(result_ids, expected);
    }

    #[test]
    fn test_label_search_with_full_label() {
        test_label_search(
            vec![
                (0, "Asteroid Field", "asteroid"),
                (1, "Asteroid Station", "station"),
            ],
            "asteroid field",
            vec![0],
        );
    }

    #[test]
    fn test_label_search_with_full_code() {
        test_label_search(
            vec![
                (0, "Asteroid Field", "asteroid"),
                (1, "Asteroid Station", "asteroid_station"),
            ],
            "asteroid",
            vec![0],
        );
    }

    #[test]
    fn test_label_search_with_partial_label() {
        test_label_search(
            vec![
                (0, "Asteroid Field", "not"),
                (1, "Asteroid Station", "station"),
            ],
            "asteroid",
            vec![0, 1],
        );
    }

    #[test]
    fn test_label_search_with_partial_code() {
        test_label_search(
            vec![
                (0, "Field", "asteroid_field"),
                (1, "Station", "asteroid_station"),
            ],
            "asteroid",
            vec![0, 1],
        );
    }
}

impl SnapshotSupport for Labels {
    fn save(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, comp) in &self.index {
            let value = json!(comp);
            snapshot.add(id.as_u32(), "label", value);
        }
    }

    fn load(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}
