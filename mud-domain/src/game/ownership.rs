use crate::errors::{Error, Result};
use commons::ObjId;
use logs::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Ownerships {
    /// key = obj, value = owner
    owners: HashMap<ObjId, ObjId>,
    /// key =owner, values = objects
    goods: HashMap<ObjId, Vec<ObjId>>,
}

impl Ownerships {
    pub fn new() -> Self {
        Ownerships {
            owners: HashMap::new(),
            goods: HashMap::new(),
        }
    }

    pub fn set_owner(&mut self, obj_id: ObjId, owner_id: ObjId) {
        self.remove_owner(obj_id);
        self.owners.insert(obj_id, owner_id);
        self.goods
            .entry(owner_id)
            .or_insert(Vec::new())
            .push(obj_id);

        debug!("{:?} is now onwed by {:?}", obj_id, owner_id);
    }

    pub fn remove_owner(&mut self, obj_id: ObjId) -> Option<ObjId> {
        let last_owner = self.owners.remove(&obj_id);
        if let Some(owner) = last_owner {
            self.goods.get_mut(&owner).unwrap().retain(|i| *i != obj_id);

            debug!("{:?} owner removed, previous owner was {:?}", obj_id, owner);
        }

        last_owner
    }

    pub fn get_owner(&self, id: ObjId) -> Option<ObjId> {
        self.owners.get(&id).cloned()
    }

    pub fn list(&self, owner_id: ObjId) -> Vec<ObjId> {
        self.goods.get(&owner_id).cloned().unwrap_or(Vec::new())
    }

    pub fn count(&self, owner_id: ObjId) -> usize {
        self.goods
            .get(&owner_id)
            .map(|list| list.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ownership() {
        let mut ownership = Ownerships::new();

        // 1 and 2 belongs to 0
        ownership.set_owner(ObjId(1), ObjId(0));
        ownership.set_owner(ObjId(2), ObjId(0));

        assert_eq!(Some(ObjId(0)), ownership.get_owner(ObjId(1)));
        assert_eq!(Some(ObjId(0)), ownership.get_owner(ObjId(2)));
        assert_eq!(vec![ObjId(1), ObjId(2)], ownership.list(ObjId(0)));

        // 2 belongs to 0, 1 belongs to 5
        ownership.set_owner(ObjId(1), ObjId(5));
        assert_eq!(Some(ObjId(5)), ownership.get_owner(ObjId(1)));
        assert_eq!(Some(ObjId(0)), ownership.get_owner(ObjId(2)));
        assert_eq!(vec![ObjId(2)], ownership.list(ObjId(0)));
        assert_eq!(vec![ObjId(1)], ownership.list(ObjId(5)));

        // 2 belongs to 0, 1 belongs to no-one
        assert_eq!(Some(ObjId(5)), ownership.remove_owner(ObjId(1)));
        assert_eq!(None, ownership.get_owner(ObjId(1)));
        assert!(ownership.list(ObjId(5)).is_empty());
    }
}
