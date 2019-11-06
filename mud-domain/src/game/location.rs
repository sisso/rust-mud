use std::collections::HashMap;
use commons::ObjId;
use logs::*;

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
        self.index.get(&obj_id).cloned().ok_or(())
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
