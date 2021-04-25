use crate::errors::{Error, Result};
use crate::game::loader::dto::StaticId;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum LootOption {
    Obj {
        prefab_id: StaticId,
        amount_min: u32,
        amount_max: u32,
        prob: f32,
    },
    And(Vec<LootOption>),
    Or(Vec<LootOption>),
}

#[derive(Clone, Debug)]
pub struct Loot {
    pub id: ObjId,

    pub options: Vec<LootOption>,
}

impl Loot {
    pub fn new(id: ObjId) -> Self {
        Loot {
            id,
            options: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Loots {
    index: HashMap<ObjId, Loot>,
}

impl Loots {
    pub fn new() -> Self {
        Loots {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, loot: Loot) -> Result<()> {
        if self.index.contains_key(&loot.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(loot.id, loot);
        Ok(())
    }

    pub fn update(&mut self, loot: Loot) -> Result<()> {
        self.index.insert(loot.id, loot);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Loot> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Loot> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Loot> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Loot> + 'a {
        self.index.values()
    }
}
