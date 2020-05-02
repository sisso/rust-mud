use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Money(pub u32);

impl Money {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for Money {
    fn from(value: u32) -> Self {
        Money(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Price {
    pub id: ObjId,
    pub buy: Money,
    pub sell: Money,
}

impl Price {
    pub fn new(id: ObjId, buy: Money, sell: Money) -> Self {
        Price { id, buy, sell }
    }
}

#[derive(Clone, Debug)]
pub struct Prices {
    index: HashMap<ObjId, Price>,
}

impl Prices {
    pub fn new() -> Self {
        Prices {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, price: Price) {
        assert!(!self.index.contains_key(&price.id));
        self.index.insert(price.id, price);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Price> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Price> {
        self.index.get(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Price> + 'a {
        self.index.values()
    }
}

impl SnapshotSupport for Prices {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, comp) in &self.index {
            let value = json!(comp);
            snapshot.add(id.as_u32(), "prices", value);
        }
    }

    fn load_snapshot(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}
