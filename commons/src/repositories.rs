use crate::ObjId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub trait ReadRepository<T> {
    fn exists(&self, id: ObjId) -> bool;

    fn get<'a>(&'a self, id: ObjId) -> Option<&'a T>;

    fn list_keys<'a>(&'a self) -> Box<dyn Iterator<Item = ObjId> + 'a>;

    fn list<'a>(&'a self) -> Box<dyn Iterator<Item = (ObjId, &'a T)> + 'a>;

    fn len(&self) -> usize;
}

pub trait WriteRepository<T> {
    fn add(&mut self, id: ObjId, obj: T) -> bool;

    fn update(&mut self, id: ObjId, obj: T);

    fn get_mut<'a>(&'a mut self, id: ObjId) -> Option<&'a mut T>;

    fn remove(&mut self, id: ObjId) -> Option<T>;

    fn clear(&mut self);
}

pub struct HashMapRepository<T> {
    map: HashMap<ObjId, T>,
}

impl<T> Default for HashMapRepository<T> {
    fn default() -> Self {
        HashMapRepository {
            map: Default::default(),
        }
    }
}

impl<T> ReadRepository<T> for HashMapRepository<T> {
    fn exists(&self, id: ObjId) -> bool {
        self.map.contains_key(&id)
    }

    fn get<'a>(&'a self, id: ObjId) -> Option<&'a T> {
        self.map.get(&id)
    }

    fn list_keys<'a>(&'a self) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        Box::new(self.map.keys().cloned())
    }

    fn list<'a>(&'a self) -> Box<dyn Iterator<Item = (ObjId, &'a T)> + 'a> {
        Box::new(self.map.iter().map(|(k, v)| (*k, v)))
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T> WriteRepository<T> for HashMapRepository<T> {
    fn add(&mut self, id: ObjId, obj: T) -> bool {
        let e = self.map.entry(id);
        match &e {
            Entry::Occupied(_) => false,
            Entry::Vacant(_) => {
                e.or_insert(obj);
                true
            }
        }
    }

    fn update(&mut self, id: ObjId, obj: T) {
        self.map.insert(id, obj);
    }

    fn get_mut<'a>(&'a mut self, id: ObjId) -> Option<&'a mut T> {
        self.map.get_mut(&id)
    }

    fn remove(&mut self, id: ObjId) -> Option<T> {
        self.map.remove(&id)
    }

    fn clear(&mut self) {
        self.map.clear()
    }
}

pub struct VecRepository<T> {
    index_by_uid: Vec<Option<usize>>,
    obj_by_index: Vec<Option<T>>,
}

impl<T> VecRepository<T> {
    // return the usize index of index_by_id
    fn get_uid(&self, id: ObjId) -> Option<usize> {
        let uid = id.0 as usize;
        if uid < self.index_by_uid.len() {
            Some(uid)
        } else {
            None
        }
    }
}

impl<T> ReadRepository<T> for VecRepository<T> {
    fn exists(&self, id: ObjId) -> bool {
        self.get_uid(id)
            .map(|i| self.index_by_uid[i].is_some())
            .unwrap_or(false)
    }

    fn get<'a>(&'a self, id: ObjId) -> Option<&'a T> {
        let index = self.get_uid(id).and_then(|uid| self.index_by_uid[uid])?;
        self.obj_by_index[index].as_ref()
    }

    fn list_keys<'a>(&'a self) -> Box<dyn Iterator<Item = ObjId> + 'a> {
        let iter = self
            .index_by_uid
            .iter()
            .enumerate()
            .filter(move |(_, index)| {
                index
                    .map(|index| self.obj_by_index[index].is_some())
                    .unwrap_or(false)
            })
            .map(|(uid, _)| ObjId(uid as u32));

        Box::new(iter)
    }

    fn list<'a>(&'a self) -> Box<dyn Iterator<Item = (ObjId, &'a T)> + 'a> {
        let iter = self
            .index_by_uid
            .iter()
            .enumerate()
            .flat_map(move |(uid, index)| {
                let obj = index.and_then(|index| self.obj_by_index[index].as_ref())?;
                Some((ObjId(uid as u32), obj))
            });

        Box::new(iter)
    }

    fn len(&self) -> usize {
        self.obj_by_index.len()
    }
}

impl<T> WriteRepository<T> for VecRepository<T> {
    fn add(&mut self, id: ObjId, obj: T) -> bool {
        if self.exists(id) {
            return false;
        }

        let index = self.obj_by_index.len();

        // resize
        let uid = id.0 as usize;
        if uid < self.index_by_uid.len() {
            // already existing
            if self.index_by_uid[uid].is_some() {
                return false;
            }
            // update new index
            self.index_by_uid[uid] = Some(index);
        } else {
            // grow the index
            while self.index_by_uid.len() < uid as usize {
                self.index_by_uid.push(None)
            }

            self.index_by_uid.push(Some(index));
        }

        // add obj
        self.obj_by_index.push(Some(obj));

        true
    }

    fn update(&mut self, id: ObjId, obj: T) {
        match self.get_uid(id).and_then(|uid| self.index_by_uid[uid]) {
            Some(index) => self.obj_by_index[index] = Some(obj),
            None => assert!(self.add(id, obj)),
        }
    }

    fn get_mut<'a>(&'a mut self, id: ObjId) -> Option<&'a mut T> {
        let uid = self.get_uid(id)?;
        let index = self.index_by_uid[uid]?;
        self.obj_by_index[index].as_mut()
    }

    fn remove(&mut self, id: ObjId) -> Option<T> {
        let uid = self.get_uid(id)?;

        // clear index
        let index = self.index_by_uid[uid].take()?;

        // remove element and swap with the last
        let latest_index = self.obj_by_index.len() - 1;
        let obj = self.obj_by_index.swap_remove(index);

        // update latest index to new place
        let latest_uid = self
            .index_by_uid
            .iter()
            .enumerate()
            .position(|(uid, i_index)| {
                i_index
                    .map(|i_index| i_index == latest_index)
                    .unwrap_or(false)
            })
            .expect("fail to find last element that replace removed object");
        self.index_by_uid[latest_uid] = Some(index);

        // done
        obj
    }

    fn clear(&mut self) {
        self.index_by_uid.clear();
        self.obj_by_index.clear();
    }
}

impl<T> Default for VecRepository<T> {
    fn default() -> Self {
        VecRepository {
            index_by_uid: vec![],
            obj_by_index: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::repositories::{HashMapRepository, ReadRepository, VecRepository, WriteRepository};
    use crate::ObjId;
    use std::io::Write;

    #[test]
    fn test_hashmaprepository() {
        let mut repo = HashMapRepository::<&str>::default();
        test_repository(&mut repo);
    }

    #[test]
    fn test_vector_repository() {
        let mut repo = VecRepository::<&str>::default();
        test_repository(&mut repo);
    }

    fn test_repository<T>(repo: &mut T)
    where
        T: ReadRepository<&'static str> + WriteRepository<&'static str>,
    {
        test_repository_basics(repo);

        repo.clear();
        assert_eq!(0, repo.len());

        test_repository_update(repo);
        test_repository_removal(repo);
    }

    fn test_repository_basics<T>(repo: &mut T)
    where
        T: ReadRepository<&'static str> + WriteRepository<&'static str>,
    {
        repo.clear();

        assert!(repo.get(ObjId(0)).is_none());
        assert!(repo.add(ObjId(0), "ok"));
        assert!(repo.get(ObjId(0)).is_some());
        assert!(!repo.add(ObjId(0), "ox"));
        assert_eq!(&"ok", repo.get(ObjId(0)).unwrap_or(&"not"));

        assert_eq!(vec![ObjId(0)], repo.list_keys().collect::<Vec<_>>());
        assert_eq!(vec![(ObjId(0), &"ok")], repo.list().collect::<Vec<_>>());
    }

    fn test_repository_removal<T>(repo: &mut T)
    where
        T: ReadRepository<&'static str> + WriteRepository<&'static str>,
    {
        repo.clear();

        assert!(repo.add(ObjId(1), "n1"));
        assert!(repo.add(ObjId(5), "n5"));
        assert!(repo.add(ObjId(3), "n3"));
        assert!(repo.add(ObjId(8), "n8"));

        assert_eq!(Some("n3"), repo.remove(ObjId(3)));
        assert_eq!(3, repo.len());

        assert_eq!(None, repo.remove(ObjId(10)));
        assert_eq!(3, repo.len());

        let ids = repo.list_keys().map(|i| i.0).collect::<Vec<_>>();
        assert!(ids.iter().position(|i| *i == 1).is_some());
        assert!(ids.iter().position(|i| *i == 5).is_some());
        assert!(ids.iter().position(|i| *i == 8).is_some());
    }

    fn test_repository_update<T>(repo: &mut T)
    where
        T: ReadRepository<&'static str> + WriteRepository<&'static str>,
    {
        repo.clear();

        // updates check
        repo.update(ObjId(1), "ok1");
        repo.update(ObjId(2), "ok2");
        repo.update(ObjId(1), "ok3");

        assert_eq!(2, repo.len());
        assert_eq!(Some(&"ok3"), repo.get(ObjId(1)));
        assert_eq!(Some(&"ok2"), repo.get(ObjId(2)));
    }
}
