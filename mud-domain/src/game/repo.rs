use commons::ObjId;

use crate::errors::{Error, Result};

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
