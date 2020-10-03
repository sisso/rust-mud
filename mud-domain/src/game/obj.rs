use crate::errors::{self, AsResult, Error, Result};
use crate::game::domain::NextId;
use crate::game::loader::dto::StaticId;
use commons::{ObjId, OBJ_ID_STATIC_RANGE};
use logs::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::Value::Object;
use std::collections::HashMap;

pub type PrefabId = StaticId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obj {
    id: ObjId,
    prefab_id: Option<PrefabId>,
}

impl Obj {
    pub fn new(id: ObjId) -> Self {
        Obj {
            id,
            prefab_id: None,
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

    pub fn insert(&mut self, id: ObjId) -> Result<()> {
        if self.objects.contains_key(&id) {
            return Err(Error::ConflictException);
        }

        debug!("{:?} obj insert", id);
        self.next_id.set_max(id.as_u32());
        self.objects.insert(id, Obj::new(id));
        Ok(())
    }

    pub fn set_prefab_id(&mut self, obj_id: ObjId, prefab_id: PrefabId) -> Result<()> {
        debug!("{:?} prefab_id set to {:?}", obj_id, prefab_id);
        let obj = self.objects.get_mut(&obj_id).as_result()?;
        obj.prefab_id = Some(prefab_id);
        Ok(())
    }

    pub fn get_prefab_id(&self, obj_id: ObjId) -> Option<PrefabId> {
        self.objects.get(&obj_id).and_then(|obj| obj.prefab_id)
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
