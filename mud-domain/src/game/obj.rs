use std::collections::HashMap;
use crate::game::domain::NextId;
use crate::game::loader::load_rooms_objects;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct ObjId(pub u32);

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
}
