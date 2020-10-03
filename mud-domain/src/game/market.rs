use crate::errors::{Error, Result};
use crate::game::tags::TagId;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MarketTrade {
    pub tags: Vec<TagId>,
    /// price mult that a vendor buy this product, this happens when a mob sell a object
    pub buy_price_mult: Option<f32>,
    /// price mult that a vendor will sell this product, this happens when a mob buy a object
    pub sell_price_mult: Option<f32>,
    // pub max_demand: Option<f32>,
    // pub change_per_cycle: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct Market {
    pub id: ObjId,
    pub trades: Vec<MarketTrade>,
}

impl Market {
    pub fn new(id: ObjId) -> Self {
        Market { id, trades: vec![] }
    }
}

#[derive(Clone, Debug)]
pub struct Markets {
    index: HashMap<ObjId, Market>,
}

impl Markets {
    pub fn new() -> Self {
        Markets {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, market: Market) -> Result<()> {
        if self.index.contains_key(&market.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(market.id, market);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Market> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Market> {
        self.index.get(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Market> + 'a {
        self.index.values()
    }
}
