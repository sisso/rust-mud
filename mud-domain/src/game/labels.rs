use crate::utils::strinput::StrInput;
use crate::utils::text;
use commons::ObjId;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::panic::resume_unwind;

pub const NO_LABEL: &str = "???";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Label {
    pub id: ObjId,
    pub label: String,
    pub desc: String,
}

impl Label {
    pub fn new(id: ObjId, label: &str) -> Self {
        Label {
            id,
            label: label.to_string(),
            desc: label.to_string(),
        }
    }

    pub fn new_desc(id: ObjId, label: &str, desc: &str) -> Self {
        Label {
            id,
            label: label.to_string(),
            desc: desc.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        log::debug!("{:?} added", label);
        self.index.insert(label.id, label);
    }

    pub fn update(&mut self, label: Label) {
        assert!(self.index.contains_key(&label.id));
        log::debug!("{:?} updated", label);
        self.index.insert(label.id, label);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Label> {
        log::debug!("{:?} removed", id);
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
            .unwrap_or(NO_LABEL)
    }

    pub fn resolve(&self, ids: &Vec<ObjId>) -> Vec<&Label> {
        ids.iter()
            .map(|&id| self.get(id).expect("label not found"))
            .collect()
    }

    pub fn resolve_labels(&self, ids: &Vec<ObjId>) -> Vec<&str> {
        ids.iter().map(|id| self.get_label_f(*id)).collect()
    }

    // list labels appending the number in case of similars
    pub fn resolve_labels_candidates(&self, ids: &Vec<ObjId>) -> Vec<String> {
        labels_for_candidates(&ids.iter().flat_map(|id| self.get(*id)).collect())
            .into_iter()
            .map(|i| i.1)
            .collect()
    }

    pub fn search(&self, ids: &Vec<ObjId>, input: &str) -> Vec<ObjId> {
        let labels = self.resolve(ids);
        label_search(&labels, input)
    }
}

pub fn labels_for_candidates(labels: &Vec<&Label>) -> Vec<(ObjId, String)> {
    // sort by label, id
    let mut lab_id: Vec<_> = labels.iter().map(|l| (l.label.as_str(), l.id)).collect();
    lab_id.sort();

    // get labels as string
    let mut lab = lab_id
        .into_iter()
        .map(|i| (i.1, i.0.to_string()))
        .collect::<Vec<(ObjId, String)>>();

    // for each sequence of same lable, append a number after the first occurrence
    let mut k = 1;
    for i in 1..lab.len() {
        if lab[i].1 == lab[i - k].1 {
            k += 1;
            lab[i] = (lab[i].0, format!("{}.{}", lab[i].1, k));
        } else {
            k = 1;
        }
    }

    lab
}

pub fn label_search<'a>(labels: &Vec<&'a Label>, input: &str) -> Vec<ObjId> {
    let candidates = labels_for_candidates(labels);

    let mut result = vec![];

    // search by exactly label
    for (id, label) in &candidates {
        if text::is_text_eq(label.as_str(), input) {
            result.push(*id);
        }
    }

    if !result.is_empty() {
        return result;
    }

    // search by fuzzy label
    for (id, label) in &candidates {
        if text::is_text_like(label.as_str(), input) {
            result.push(*id);
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_label_search(labels: Vec<(u32, &str)>, input: &str, expected: Vec<u32>) {
        let labels: Vec<Label> = labels
            .into_iter()
            .map(|(id, label)| Label {
                id: ObjId(id),
                label: label.to_string(),
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
            vec![(0, "Asteroid Field"), (1, "Asteroid Station")],
            "asteroid field",
            vec![0],
        );
    }

    #[test]
    fn test_label_search_with_partial_label() {
        test_label_search(
            vec![(0, "Asteroid Field"), (1, "Asteroid Station")],
            "asteroid",
            vec![0, 1],
        );
    }

    #[test]
    fn test_label_search_with_id() {
        let labels = vec![
            Label {
                id: 0.into(),
                label: "Obj A".to_string(),
                desc: "".to_string(),
            },
            Label {
                id: 1.into(),
                label: "Obj B".to_string(),
                desc: "".to_string(),
            },
            Label {
                id: 2.into(),
                label: "Obj B".to_string(),
                desc: "".to_string(),
            },
            Label {
                id: 3.into(),
                label: "Obj B".to_string(),
                desc: "".to_string(),
            },
        ];

        let labels_ref = labels.iter().collect();
        let found = super::label_search(&labels_ref, "obj b.2");
        assert_eq!(1, found.len());
        assert_eq!(ObjId(2), found[0]);
    }

    #[test]
    fn test_labels_for_candidates() {
        let labels = vec![
            Label {
                id: 0.into(),
                label: "ObjA".to_string(),
                desc: "".to_string(),
            },
            Label {
                id: 2.into(),
                label: "ObjB".to_string(),
                desc: "".to_string(),
            },
            Label {
                id: 1.into(),
                label: "ObjB".to_string(),
                desc: "".to_string(),
            },
            Label {
                id: 3.into(),
                label: "ObjB".to_string(),
                desc: "".to_string(),
            },
        ];

        let labels_ref = labels.iter().collect();
        let labels_str = super::labels_for_candidates(&labels_ref);
        assert_eq!(
            vec![
                (0.into(), "ObjA".to_string()),
                (1.into(), "ObjB".to_string()),
                (2.into(), "ObjB.2".to_string()),
                (3.into(), "ObjB.3".to_string())
            ],
            labels_str
        );
    }
}
