use crate::errors::Error::Conflict;
use crate::errors::{self, Error};
use crate::game::domain::NextId;
use crate::game::loader::StaticId;
use commons::ObjId;
use logs::*;
use std::collections::HashMap;

pub const NAMESPACE_RESERVED: u32 = 100000;

#[derive(Clone, Debug)]
pub struct Obj {
    id: ObjId,
    static_id: Option<StaticId>,
}

impl Obj {
    pub fn new(id: ObjId) -> Self {
        Obj {
            id,
            static_id: None,
        }
    }
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
        self.objects.insert(id, Obj::new(id));
        id
    }

    pub fn insert(&mut self, id: ObjId) -> errors::Result<()> {
        if self.objects.contains_key(&id) {
            return Err(Conflict);
        }

        debug!("{:?} obj insert", id);
        self.next_id.set_max(id.as_u32());
        self.objects.insert(id, Obj::new(id));
        Ok(())
    }

    pub fn set_static_id(&mut self, id: ObjId, static_id: StaticId) -> errors::Result<()> {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.static_id = Some(static_id);
            debug!("{:?} obj update static_id to {:?}", id, static_id);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn get_static_id(&self, id: ObjId) -> Option<StaticId> {
        self.objects.get(&id).and_then(|obj| obj.static_id)
    }

    /// Make sure you remove from everything else first
    pub fn remove(&mut self, obj_id: ObjId) -> bool {
        if self.objects.remove(&obj_id).is_some() {
            debug!("{:?} obj removed ", obj_id);
            true
        } else {
            false
        }
    }

    pub fn exists(&self, obj_id: ObjId) -> bool {
        return self.objects.contains_key(&obj_id);
    }

    pub fn list<'a>(&'a mut self) -> impl Iterator<Item = &ObjId> + 'a {
        self.objects.keys()
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
