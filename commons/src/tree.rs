use std::collections::{HashMap};
use std::hash::Hash;

// TODO: replace by 2 hashmap key -> value, key -> parent_key
//#[derive(Debug, Clone)]
//pub enum Error {
//   IndexConflict,
//   ParentNotFound,
//}
//
//struct TreeNode<K, V> {
//   parent: Option<K>,
//   value: V,
//}
//
//pub struct Tree<K, V> {
//   index: HashMap<K, TreeNode<K, V>>,
//}
//
//impl<K: Hash + Eq, V> Tree<K, V> {
//   pub fn new() -> Self {
//      Tree {
//         index: HashMap::new(),
//      }
//   }
//
//   pub fn insert(&mut self, key: K, value: V, parent: Option<K>) -> Result<(), Error> {
//      if let Some(parent_id) = &parent {
//         if !self.index.contains_key(parent_id) {
//            return Err(Error::ParentNotFound);
//         }
//      }
//
//      let node = TreeNode {
//         parent,
//         value,
//      };
//
//      match self.index.insert(key, node) {
//         Some(_) => Err(Error::IndexConflict),
//         None => Ok(()),
//      }
//   }
//
//   pub fn get(&self, key: &K) -> Option<&V> {
//      self.index.get(key)
//          .map(|i| &i.value)
//   }
//
//   pub fn children(&self, key: &K) -> Vec<&K> {
//       self.index.iter()
//           .filter(|(_, value)| {
//              value.parent
//                  .as_ref()
//                  .map(|parent_id| parent_id == key)
//                  .unwrap_or(false)
//           })
//           .map(|(id, _)| id)
//           .collect()
//   }
//}
//
//
//#[test]
//fn test_tree() {
//   use std::collections::{HashSet};
//   let mut tree = Tree::new();
//
//   tree.insert(0, "Sun", None).unwrap();
//   tree.insert(1, "Earth", Some(0)).unwrap();
//   tree.insert(2, "Moon", Some(1)).unwrap();
//   tree.insert(3, "Venus", Some(0)).unwrap();
//
//   assert_eq!(tree.get(&0).unwrap(), &"Sun");
//   assert_eq!(tree.get(&1).unwrap(), &"Earth");
//   assert_eq!(tree.get(&2).unwrap(), &"Moon");
//   assert_eq!(tree.get(&3).unwrap(), &"Venus");
//
//   let children_set: HashSet<&i32> = tree.children(&0).into_iter().collect();
//   assert_eq!(children_set, vec![&1, &3].into_iter().collect());
//   assert_eq!(tree.children(&1), vec![&2]);
//   assert!(tree.children(&2).is_empty());
//}
