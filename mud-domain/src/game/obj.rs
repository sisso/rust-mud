use std::collections::HashMap;
use crate::game::domain::NextId;
use crate::game::loader::load_rooms_objects;
use commons::ObjId;
use logs::*;

#[derive(Clone,Debug)]
pub struct Obj {
    id: ObjId,
}

pub struct Objects {
    next_id: NextId,
    objects: HashMap<ObjId, Obj>,
}

impl Objects {
    pub fn new() -> Self {
        Objects {
            next_id: NextId::new(),
            objects: HashMap::new(),
        }
    }

    pub fn insert(&mut self) -> ObjId {
        let id = ObjId(self.next_id.next());
        debug!("{:?} obj added", id);
        self.objects.insert(id, Obj {
            id
        });
        id
    }

    /// Insert an already existent ID. Dangerous operations
    pub fn insert_static(&mut self, id: ObjId) {
        assert!(!self.objects.contains_key(&id));
        debug!("{:?} obj added static", id);
        self.next_id.set_max(id.as_u32());
        self.objects.insert(id, Obj {
            id
        });
    }

    /// Make sure you remove from everything else first
    pub fn remove(&mut self, obj_id: ObjId) {
        if self.objects.remove(&obj_id).is_some() {
            debug!("{:?} obj removed ", obj_id);
        }
    }
}
