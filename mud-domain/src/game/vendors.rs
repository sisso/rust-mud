use crate::game::snapshot::{Snapshot, SnapshotSupport};
use commons::ObjId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl SnapshotSupport for Vendors {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, comp) in &self.index {
            if id.is_static() {
                continue;
            }
            let value = json!(comp);
            snapshot.add(id.as_u32(), "vendor", value);
        }
    }
}
