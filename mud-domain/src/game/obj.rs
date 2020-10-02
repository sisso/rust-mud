use crate::errors::{self, Error};
use crate::game::domain::NextId;
use crate::game::loader::dto::StaticId;
use commons::{ObjId, OBJ_ID_STATIC_RANGE};
use logs::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::Value::Object;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obj {
    id: ObjId,
}

impl Obj {
    pub fn new(id: ObjId) -> Self {
        Obj { id }
    }
}

#[derive(Debug, Clone)]
pub struct Objects {
    next_id: NextId,
    objects: HashMap<ObjId, Obj>,
}

impl Objects {
    pub fn new() -> Self {
        Objects {
            next_id: NextId::new_from(OBJ_ID_STATIC_RANGE),
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
            return Err(Error::ConflictException);
        }

        debug!("{:?} obj insert", id);
        self.next_id.set_max(id.as_u32());
        self.objects.insert(id, Obj::new(id));
        Ok(())
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

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.objects.keys()
    }
}
