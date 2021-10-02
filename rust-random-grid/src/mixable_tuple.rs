use std::cmp::Ordering;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Clone)]
pub struct MixableTuple<T>
where
    T: PartialOrd + Clone + Debug,
{
    a: T,
    b: T,
}

impl<T> MixableTuple<T>
where
    T: PartialOrd + Clone + Debug,
{
    pub fn new(v1: T, v2: T) -> Self {
        if v1.partial_cmp(&v2) == Some(Ordering::Greater) {
            MixableTuple { a: v1, b: v2 }
        } else {
            MixableTuple { a: v2, b: v1 }
        }
    }

    pub fn get_a(&self) -> &T {
        &self.a
    }

    pub fn get_b(&self) -> &T {
        &self.b
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::infinite_grid::print_rooms;
    use std::collections::HashSet;

    #[test]
    fn test_hash_and_equals() {
        let a = MixableTuple::new(3, 4);
        let b = MixableTuple::new(4, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn test_set() {
        let mut s: HashSet<MixableTuple<i32>> = Default::default();
        s.insert(MixableTuple::new(3, 4));
        assert!(s.contains(&MixableTuple::new(3, 4)));
        assert!(s.contains(&MixableTuple::new(4, 3)));
    }

    // #[test]
    // fn test_ref_mix_equal_mix_of_ref() {
    //     let a = MixableTuple::new(3, 4);
    //     let ref_of_mix = &a;
    //
    //     let mix_of_ref = MixableTuple::new(&4, &3);
    //
    //     assert_eq!(ref_of_mix, &mix_of_ref);
    // }
}
