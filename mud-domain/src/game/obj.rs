use std::collections::HashMap;
use crate::game::domain::NextId;
use crate::game::loader::load_rooms_objects;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct ObjId(pub u32);

impl ObjId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

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
        self.objects.insert(id, Obj {
            id
        });
        id
    }

    /// Insert an already existent ID. Dangerous operations
    pub fn insert_static(&mut self, id: ObjId) {
        assert!(!self.objects.contains_key(&id));
        self.next_id.set_max(id.as_u32());
        self.objects.insert(id, Obj {
            id
        });
    }
}
