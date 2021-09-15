use crate::ObjId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub trait ReadRepository<T> {
    fn exists(&self, id: ObjId) -> bool;

    fn get<'a>(&'a self, id: ObjId) -> Option<&'a T>;

    fn list_keys<'a>(&'a self) -> Box<dyn Iterator<Item = &'a ObjId> + 'a>;

    fn list<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a ObjId, &'a T)> + 'a>;
}

pub trait WriteRepository<T> {
    fn add(&mut self, id: ObjId, obj: T) -> bool;

    fn update(&mut self, id: ObjId, obj: T);

    fn get_mut<'a>(&'a mut self, id: ObjId) -> Option<&'a mut T>;

    fn remove(&mut self, id: ObjId) -> Option<T>;
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

    fn list_keys<'a>(&'a self) -> Box<dyn Iterator<Item = &'a ObjId> + 'a> {
        Box::new(self.map.keys())
    }

    fn list<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a ObjId, &'a T)> + 'a> {
        Box::new(self.map.iter())
    }
}

impl<T> WriteRepository<T> for HashMapRepository<T> {
    fn add(&mut self, id: ObjId, obj: T) -> bool {
        let mut e = self.map.entry(id);
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
}

#[cfg(test)]
mod test {
    use crate::repositories::{HashMapRepository, ReadRepository, WriteRepository};
    use crate::ObjId;
    use std::collections::HashMap;

    #[test]
    fn test_one() {
        let mut repo = HashMapRepository::<&str>::default();
        assert!(repo.get(ObjId(0)).is_none());
        assert!(repo.add(ObjId(0), "ok"));
        assert!(repo.get(ObjId(0)).is_some());
        assert!(!repo.add(ObjId(0), "ok"));
        assert_eq!(&"ok", repo.get(ObjId(0)).unwrap_or(&"not"));

        assert_eq!(
            vec![ObjId(0)],
            repo.list_keys().cloned().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![(ObjId(0), &"ok")],
            repo.list().map(|(k, v)| (k.clone(), v)).collect::<Vec<_>>()
        );
    }
}
