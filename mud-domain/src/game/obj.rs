use std::collections::HashMap;
use crate::game::domain::NextId;
use commons::ObjId;
use logs::*;

pub const NAMESPACE_RESERVED: u32 = 100000;

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
            next_id: NextId::new_from(NAMESPACE_RESERVED),
            objects: HashMap::new(),
        }
    }

    pub fn create(&mut self) -> ObjId {
        let id = ObjId(self.next_id.next());
        debug!("{:?} created", id);
        self.objects.insert(id, Obj {
            id
        });
        id
    }

    pub fn insert(&mut self, id: ObjId) {
        assert!(!self.objects.contains_key(&id));
        debug!("{:?} obj insert", id);
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

// TODO: use or remove?
pub trait ReadRepository<'a, T> {
    fn exists(id: ObjId) -> bool;

    fn get(id: ObjId) -> Option<&'a T>;
}

// TODO: use or remove?
pub trait WriteRepository<T> {
    fn add(obj: T);

    fn remove(id: ObjId) -> bool;
}
