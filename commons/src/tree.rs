use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Tree<K: Hash + Eq + Copy + Clone> {
    parents: HashMap<K, K>,
}

impl<K: Hash + Eq + Copy + Clone> Tree<K> {
    pub fn new() -> Self {
        Tree {
            parents: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, parent: K) -> Option<K> {
        self.parents.insert(key, parent)
    }

    pub fn remove(&mut self, key: K) -> Option<K> {
        self.parents.remove(&key)
    }

    pub fn get(&self, key: K) -> Option<K> {
        self.parents.get(&key).cloned()
    }

    pub fn children<'a>(&'a self, root: K) -> impl Iterator<Item = K> + 'a {
        self.parents
            .iter()
            .filter(move |(_, &value)| value == root)
            .map(|(&key, _)| key)
    }

    pub fn children_deep(&self, root: K) -> Vec<K> {
        let mut buffer = Vec::new();

        for i in self.children(root) {
            buffer.push(i);

            let childrens = self.children_deep(i);
            buffer.extend(childrens);
        }

        buffer
    }

    pub fn parents(&self, from: K) -> Vec<K> {
        let mut buffer = vec![];
        let mut current = from;
        loop {
            let parent = self.get(current);
            match parent {
                Some(location_id) => {
                    buffer.push(location_id);
                    current = location_id;
                }
                None => break,
            }
        }
        buffer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sample_tree() -> Tree<i32> {
        let mut tree = Tree::new();
        /*
           0
           +1
           |+2
           ||+5
           | |+6
           |+4
           ||+7
           +3
        */
        tree.insert(1, 0);
        tree.insert(2, 1);
        tree.insert(3, 0);
        tree.insert(4, 1);
        tree.insert(5, 2);
        tree.insert(6, 5);
        tree.insert(7, 4);
        tree
    }

    #[test]
    fn test_tree_children() {
        let tree = sample_tree();

        assert_eq!(tree.get(0), None);
        assert_eq!(tree.get(1), Some(0));
        assert_eq!(tree.get(2), Some(1));
        assert_eq!(tree.get(3), Some(0));

        let mut children: Vec<_> = tree.children(0).collect();
        children.sort();
        assert_eq!(children, vec![1, 3]);
        assert_eq!(tree.children(4).collect::<Vec<_>>(), vec![7]);
        assert!(tree.children(7).next().is_none());
    }

    #[test]
    fn test_tree_children_deep() {
        let tree = sample_tree();

        let tests = vec![
            (0, vec![1, 2, 3, 4, 5, 6, 7]),
            (1, vec![2, 4, 5, 6, 7]),
            (2, vec![5, 6]),
            (3, vec![]),
            (4, vec![7]),
            (5, vec![6]),
            (6, vec![]),
            (7, vec![]),
        ];

        for (index, expected) in tests {
            let mut children = tree.children_deep(index);
            children.sort();
            assert_eq!(children, expected);
        }
    }

    #[test]
    fn test_tree_parents() {
        let tree = sample_tree();

        let tests = vec![
            (0, vec![]),
            (1, vec![0]),
            (2, vec![1, 0]),
            (3, vec![0]),
            (4, vec![1, 0]),
            (5, vec![2, 1, 0]),
            (6, vec![5, 2, 1, 0]),
            (7, vec![4, 1, 0]),
        ];

        for (index, expected) in tests {
            let children = tree.parents(index);
            assert_eq!(children, expected);
        }
    }
}
