use commons::ObjId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Vendor {
    pub id: ObjId,
}

impl Vendor {
    pub fn new(id: ObjId) -> Self {
        Vendor { id }
    }
}

#[derive(Clone, Debug)]
pub struct Vendors {
    index: HashMap<ObjId, Vendor>,
}

impl Vendors {
    pub fn new() -> Self {
        Vendors {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, vendor: Vendor) {
        assert!(!self.index.contains_key(&vendor.id));
        self.index.insert(vendor.id, vendor);
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Vendor> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Vendor> {
        self.index.get(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }
}
