use crate::errors::{Error, Result};
use crate::game::item::Weight;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub id: ObjId,
    pub max_weight: Option<Weight>,
    pub current_weight: Option<Weight>,
}

impl Inventory {
    pub fn new(id: ObjId) -> Self {
        Inventory {
            id: id,
            current_weight: None,
            max_weight: None,
        }
    }

    pub fn can_add(&self, weight: Weight) -> bool {
        self.max_weight
            .as_ref()
            .map(|&max_weight| self.current_weight.unwrap_or(0.0) + weight <= max_weight)
            .unwrap_or(true)
    }
}

#[derive(Clone, Debug)]
pub struct Inventories {
    index: HashMap<ObjId, Inventory>,
}

impl Inventories {
    pub fn new() -> Self {
        Inventories {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, inventory: Inventory) -> Result<()> {
        if self.index.contains_key(&inventory.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(inventory.id, inventory);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Inventory> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Inventory> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Inventory> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Inventory> + 'a {
        self.index.values()
    }
}
