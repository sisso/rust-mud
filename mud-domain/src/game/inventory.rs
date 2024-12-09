use crate::errors::{Error, Result};
use crate::game::item::Weight;
use crate::game::loader::dto::{CanLoad, ObjData};
use crate::game::loader::LoadingCtx;
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn available(&self) -> Weight {
        self.max_weight.unwrap_or(0.0) - self.current_weight.unwrap_or(0.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn get_max_weight(&self, id: ObjId) -> Option<Weight> {
        self.index.get(&id).and_then(|i| i.max_weight)
    }
}

impl CanLoad for Inventories {
    fn load(&mut self, references: &LoadingCtx, obj_id: ObjId, data: &ObjData) -> Result<()> {
        if let Some(inventory_data) = &data.inventory {
            let mut inv = Inventory::new(obj_id);
            inv.max_weight = inventory_data.max_weight;
            self.add(inv).unwrap();
        }

        Ok(())
    }
}
