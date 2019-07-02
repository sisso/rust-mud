use std::collections::HashMap;

struct Index<T> {
    next_id: u32,
    index: HashMap<u32, T>
}

impl <T> Index<T> {
    fn new() -> Self {
        Index {
            next_id: 0,
            index: HashMap::new(),
        }
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn add(&mut self, id: u32, obj: T) {
        self.index.insert(id, obj);
    }

    fn remove(&mut self, id: &u32) {
        self.index.remove(&id);
    }

    fn get(&self, id: &u32) -> &T {
        self.index.get(id).expect(format!("could not find object with id {}", id).as_str())
    }

    fn find(&self, id: &u32) -> Option<&T> {
        self.index.get(id)
    }

    // TODO: return iterator
    fn list(&self) -> Vec<&u32> {
        self.index.keys().collect()
    }
}
