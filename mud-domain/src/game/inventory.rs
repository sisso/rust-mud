use crate::errors::{Error, Result};
use crate::game::item::Weight;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Inventory {
    pub id: ObjId,
    pub max_weight: Weight,
    pub current_weight: Weight,
}

impl Inventory {
    pub fn new(id: ObjId, current: Weight, max: Weight) -> Self {
        Inventory {
            id: id,
            current_weight: current,
            max_weight: max,
        }
    }

    pub fn can_add(&self, weight: Weight) -> bool {
        self.current_weight + weight <= self.max_weight
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
