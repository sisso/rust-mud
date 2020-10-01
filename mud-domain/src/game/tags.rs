use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct TagId(usize);

pub struct Tag(String);

#[derive(Clone, Debug)]
pub struct Tags {
    index: HashMap<ObjId, HashSet<TagId>>,
    tags: Vec<(TagId, String)>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            index: HashMap::new(),
            tags: vec![],
        }
    }

    pub fn get_id(&mut self, value: &str) -> TagId {
        match self.tags.iter().find(|(id, s)| s.as_str() == value) {
            Some((id, _)) => *id,
            None => {
                let tag_id = TagId(self.tags.len());
                self.tags.push((tag_id, value.to_string()));
                tag_id
            }
        }
    }

    pub fn get_str(&self, tag_id: TagId) -> Option<&str> {
        self.tags
            .iter()
            .find(|(id, s)| *id == tag_id)
            .map(|(_, s)| s.as_str())
    }

    pub fn add(&mut self, obj_id: ObjId, tag_id: TagId) {
        let tags = self.index.entry(obj_id).or_default();
        tags.insert(tag_id);
    }

    pub fn get_tags(&self, obj_id: ObjId) -> Option<&HashSet<TagId>> {
        self.index.get(&obj_id)
    }

    pub fn has(&self, obj_id: ObjId, tag_id: TagId) -> bool {
        self.index
            .get(&obj_id)
            .map(|tags| tags.contains(&tag_id))
            .unwrap_or(false)
    }

    pub fn resolve_str(&self, tags: &Vec<TagId>) -> Option<Vec<&str>> {
        tags.iter().map(|tag| self.get_str(*tag)).collect()
    }

    pub fn resolve_strings(&self, tags: &Vec<TagId>) -> Option<Vec<String>> {
        self.resolve_str(tags)
            .map(|i| i.iter().map(|s| s.to_string()).collect())
    }

    pub fn resolve_tags(&mut self, tags: &Vec<&str>) -> Vec<TagId> {
        tags.iter().map(|tag_str| self.get_id(tag_str)).collect()
    }
}
