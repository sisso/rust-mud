use crate::errors::{Error, Result};
use crate::game::loader::dto::{CanLoad, CanSnapshot, ObjData};
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelingConnection {
    pub zone_id: ObjId,
    pub distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Travel {
    pub id: ObjId,
    pub connections: Vec<TravelingConnection>,
}

impl Travel {
    pub fn new(id: ObjId) -> Self {
        Travel {
            id,
            connections: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Travels {
    index: HashMap<ObjId, Travel>,
}

impl Travels {
    pub fn new() -> Self {
        Travels {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, travel: Travel) -> Result<()> {
        if self.index.contains_key(&travel.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(travel.id, travel);
        Ok(())
    }

    pub fn update(&mut self, travel: Travel) -> Result<()> {
        self.index.insert(travel.id, travel);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Travel> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Travel> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Travel> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Travel> + 'a {
        self.index.values()
    }
}

impl CanLoad for Travels {
    fn load(&mut self, obj_id: ObjId, data: &ObjData) -> Result<()> {
        if let Some(travel_data) = &data.travel {
            let mut travel_data = travel_data.clone();
            travel_data.id = obj_id;
            self.update(travel_data)
        } else {
            Ok(())
        }
    }
}

impl CanSnapshot for Travels {
    fn snapshot(&self, _obj_id: ObjId, _data: &mut ObjData) -> Result<()> {
        todo!()
    }
}
