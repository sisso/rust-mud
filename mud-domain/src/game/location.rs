use std::collections::HashMap;
use commons::ObjId;
use logs::*;
use crate::game::labels::Labels;
use crate::game::room::RoomId;

pub type LocationId = ObjId;

#[derive(Clone,Debug)]
pub struct Locations {
    // TODO: add inverse index
    index: HashMap<ObjId, ObjId>,
}

impl Locations {
    pub fn new() -> Self {
        Locations {
            index: HashMap::new(),
        }
    }

    pub fn set(&mut self, obj_id: ObjId, location: ObjId) {
        info!("{:?} set location {:?}", obj_id, location);
        self.index.insert(obj_id, location);
    }

    pub fn remove(&mut self, obj_id: ObjId) {
        info!("{:?} remove location", obj_id);
        self.index.remove(&obj_id);
    }

    pub fn get(&self, obj_id: ObjId) -> Result<ObjId, ()> {
        let v = self.index.get(&obj_id).cloned().ok_or(());
        v
    }

    pub fn list_at<'a>(&'a self, location_id: ObjId) -> impl Iterator<Item = ObjId> + 'a {
        self.index.iter().filter_map(move |(obj_id, loc_id)| {
            if location_id == *loc_id {
                Some(*obj_id)
            } else {
                None
            }
        })
    }
}

pub fn search_at(labels: &Labels, locations: &Locations, location_id: LocationId, input: &str) -> Vec<ObjId> {
    let candidates = locations.list_at(location_id).collect::<Vec<_>>();
    labels.search_codes(&candidates, input)
}

