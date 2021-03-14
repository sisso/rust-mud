use commons::ObjId;

use crate::errors::{Error, Result};
use std::any::TypeId;

pub trait RepoRemove<T> {
    fn remove(&mut self, id: ObjId) -> Option<T>;
}

pub trait RepoAdd<T> {
    fn add(&mut self, template: T) -> Result<()>;
}

pub trait RepoGet<T> {
    fn get(&self, id: ObjId) -> Option<&T>;
    fn get_mut(&mut self, id: ObjId) -> Option<&mut T>;
    fn exist(&self, id: ObjId) -> bool;
}

pub trait RepoList<T, B, C> {
    fn list_ids<'a>(&'a self) -> &'a B;
    fn list<'a>(&'a self) -> &'a C;
}

type Gen = u16;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Id {
    index: usize,
    gen: Gen,
}

trait HasId {
    fn get_id(&self) -> Id;
}

impl Id {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn gen(&self) -> Gen {
        self.gen
    }
}

pub struct IndexMap<T> {
    index: Vec<Option<(Id, T)>>,
}

impl<T> Default for IndexMap<T> {
    fn default() -> Self {
        IndexMap {
            index: Default::default(),
        }
    }
}

impl<T> IndexMap<T> {
    pub fn insert(&mut self, id: Id, value: T) {
        while self.index.len() <= id.index {
            self.index.push(None);
        }

        self.index[id.index] = Some((id, value));
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        if self.index.len() > id.index
            && self.index[id.index]
                .iter()
                .map(|(i_id, _)| i_id.gen == id.gen)
                .next()
                .unwrap_or(false)
        {
            let mut local: Option<(Id, T)> = None;
            std::mem::swap(&mut self.index[id.index], &mut local);
            local.map(|(_, value)| value)
        } else {
            None
        }
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        if self.index.len() > id.index {
            match &self.index[id.index] {
                Some((i_id, value)) if i_id.gen == id.gen => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        if self.index.len() > id.index {
            match &mut self.index[id.index] {
                Some((i_id, value)) if i_id.gen == id.gen => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn list(&self) -> impl Iterator<Item = &T> {
        self.index.iter().flat_map(|i| i).map(|(_id, value)| value)
    }
}

pub struct IdRepo {
    next_index: usize,
    active: Vec<Id>,
    free: Vec<Id>,
}

impl Default for IdRepo {
    fn default() -> Self {
        IdRepo {
            next_index: 0,
            active: vec![],
            free: vec![],
        }
    }
}

impl IdRepo {
    pub fn next(&mut self) -> Id {
        match self.free.pop() {
            Some(mut id) => {
                id.gen = id.gen + 1;
                self.active.push(id);
                id
            }

            None => {
                let id = Id {
                    index: self.next_index,
                    gen: 0,
                };

                self.next_index += 1;

                self.active.push(id);
                id
            }
        }
    }

    // Check if id was removed is not implemented
    pub fn remove(&mut self, id: Id) -> bool {
        self.active.retain(|i| i.index != id.index);
        self.free.push(id);
        true
    }

    pub fn list(&self) -> impl Iterator<Item = &Id> {
        self.active.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id_repo() {
        let mut repo = IdRepo::default();

        let id1 = repo.next();
        let id2 = repo.next();
        assert_eq!(vec![&id1, &id2], repo.list().collect::<Vec<_>>());

        repo.remove(id1);
        assert_eq!(vec![&id2], repo.list().collect::<Vec<_>>());

        let id3 = repo.next();
        assert_ne!(id1, id3);
        assert_eq!(id1.index(), id3.index());
        assert_ne!(id1.gen(), id3.gen());
        assert_eq!(vec![&id2, &id3], repo.list().collect::<Vec<_>>());
    }

    #[test]
    fn test_indexmap() {
        let mut repo = IdRepo::default();
        let mut map = IndexMap::default();

        // create empty ids to check if everything will work without index 0
        let _ = repo.next();
        let _ = repo.next();
        let _ = repo.next();
        let _ = repo.next();

        // insert 2 entries
        let id1 = repo.next();
        map.insert(id1, "1");

        let id2 = repo.next();
        map.insert(id2, "2");

        assert_eq!(Some(&"1"), map.get(id1));
        assert_eq!(Some(&"2"), map.get(id2));
        assert_eq!(vec![&"1", &"2"], map.list().collect::<Vec<_>>());

        // remove first item
        assert!(repo.remove(id1));
        assert_eq!(Some("1"), map.remove(id1));
        assert!(map.get(id1).is_none());
        assert_eq!(vec![&"2"], map.list().collect::<Vec<_>>());

        // insert new id with same index as previous one
        let id3 = repo.next();
        assert_eq!(id3.index(), id1.index());

        map.insert(id3, "3");
        assert!(map.get(id1).is_none());
        assert_eq!(Some(&"2"), map.get(id2));
        assert_eq!(Some(&"3"), map.get(id3));
        assert_eq!(vec![&"3", &"2"], map.list().collect::<Vec<_>>());
    }
}
