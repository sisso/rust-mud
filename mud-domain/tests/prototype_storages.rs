use commons::ObjId;

// TODO: use or remove?
pub trait ReadRepository<'a, T> {
    fn exists(&self, id: ObjId) -> bool;

    fn get(&self, id: ObjId) -> Option<&'a T>;

    // fn list<'b>(&self) -> impl Iterator<Item = &ObjId> + 'b;
}

// TODO: use or remove?
pub trait WriteRepository<T> {
    fn add(&mut self, obj: T);

    fn remove(&mut self, id: ObjId) -> bool;
}

// pub fn join_all<A, B>(repo1: &dyn ReadRepository<A>, repo2: &dyn ReadRepository<B>) -> Option<(&A, &B)> {
//     unimplemented!()
// }

// pub fn join<'a, A, B>(id: ObjId, repo1: &'a dyn ReadRepository<A>, repo2: &'a dyn ReadRepository<B>) -> Option<(&'a A, &'a B)> {
//     let a = repo1.get(id)?;
//     let b = repo2.get(id)?;
//     Some((a, b))
// }
