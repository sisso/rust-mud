use crate::errors::{Error, Result};
use crate::game::loader::dto::{CanLoad, CanSnapshot, ObjData};
use crate::game::loader::LoadingCtx;
use crate::game::zone::ZoneId;
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
    pub connections: Vec<TravelingConnection>,
}

impl Travel {
    pub fn new() -> Self {
        Travel {
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

    pub fn add(&mut self, id: ObjId, travel: Travel) -> Result<()> {
        if self.index.contains_key(&id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(id, travel);
        Ok(())
    }

    pub fn update(&mut self, id: ObjId, travel: Travel) -> Result<()> {
        self.index.insert(id, travel);
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
    fn load(&mut self, references: &LoadingCtx, obj_id: ObjId, data: &ObjData) -> Result<()> {
        if let Some(travel_data) = &data.travel {
            let mut travel_data = travel_data.clone();
            for c in &mut travel_data.connections {
                c.zone_id = references.id_map[&(c.zone_id.into())];
            }
            self.update(obj_id, travel_data)
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
