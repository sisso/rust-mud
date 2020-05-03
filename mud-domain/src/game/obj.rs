use crate::errors::{self, Error};
use crate::game::domain::NextId;
use crate::game::loader::StaticId;
use commons::save::{Snapshot, SnapshotSupport};
use commons::{ObjId, OBJ_ID_STATIC_RANGE};
use logs::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn set_static_id(&mut self, id: ObjId, static_id: StaticId) -> errors::Result<()> {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.static_id = Some(static_id);
            debug!("{:?} obj update static_id to {:?}", id, static_id);
            Ok(())
        } else {
            Err(Error::NotFoundFailure)
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

impl SnapshotSupport for Objects {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        for (id, obj) in &self.objects {
            if id.is_static() {
                continue;
            }
            snapshot.add(id.as_u32(), "object", json!(obj));
        }
    }
}
