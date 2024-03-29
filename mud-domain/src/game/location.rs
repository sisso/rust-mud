use crate::errors::AsResult;
use crate::game::labels::Labels;
use commons::tree::Tree;
use commons::ObjId;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type LocationId = ObjId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Locations {
    index: Tree<ObjId>,
}

impl Locations {
    pub fn new() -> Self {
        Locations { index: Tree::new() }
    }

    pub fn set(&mut self, obj_id: ObjId, location_id: LocationId) {
        log::debug!("{:?} set location {:?}", obj_id, location_id);
        if obj_id == location_id {
            panic!("object location can not be itself for obj_id {:?}", obj_id);
        }
        self.index.insert(obj_id, location_id);
    }

    pub fn remove(&mut self, obj_id: ObjId) {
        log::debug!("{:?} remove location", obj_id);
        self.index.remove(obj_id);
    }

    pub fn get(&self, obj_id: ObjId) -> Option<ObjId> {
        self.index.get(obj_id)
    }

    pub fn list_at<'a>(&'a self, location_id: ObjId) -> impl Iterator<Item = ObjId> + 'a {
        self.index.children(location_id)
    }

    pub fn list_deep_at(&self, location_id: LocationId) -> Vec<ObjId> {
        self.index.children_deep(location_id)
    }

    pub fn list_parents(&self, obj_id: ObjId) -> Vec<LocationId> {
        self.index.parents(obj_id)
    }

    pub fn list_parents_inclusive(&self, obj_id: ObjId) -> Vec<LocationId> {
        self.index.parents_inclusive(obj_id)
    }

    pub fn is_same_location(&self, obj_id1: ObjId, obj_id2: ObjId) -> bool {
        self.get(obj_id1) == self.get(obj_id2)
    }
}

pub fn search_at(
    labels: &Labels,
    locations: &Locations,
    location_id: LocationId,
    input: &str,
) -> Vec<ObjId> {
    let candidates = locations.list_at(location_id).collect::<Vec<_>>();
    labels.search(&candidates, input)
}

#[cfg(test)]
mod test {
    use crate::game::location::Locations;
    use commons::ObjId;
    use std::collections::HashSet;

    #[test]
    fn test_list_deep_at() {
        let locations = create_scenery();

        assert(
            locations.list_deep_at(ObjId(0)),
            vec![ObjId(1), ObjId(2), ObjId(3), ObjId(4), ObjId(5), ObjId(6)],
        );

        assert(
            locations.list_deep_at(ObjId(2)),
            vec![ObjId(4), ObjId(5), ObjId(6)],
        );

        assert(locations.list_deep_at(ObjId(5)), vec![ObjId(6)]);
        assert(locations.list_deep_at(ObjId(6)), vec![]);
    }

    #[test]
    fn test_list_parents() {
        let locations = create_scenery();

        assert(locations.list_parents(ObjId(5)), vec![ObjId(2), ObjId(0)]);
    }

    fn create_scenery() -> Locations {
        let mut locations = Locations::new();
        /*

        0
        - 1
        - 2
         - 4
         - 5
          - 6
        - 3

        */
        locations.set(ObjId(1), ObjId(0));
        locations.set(ObjId(2), ObjId(0));
        locations.set(ObjId(3), ObjId(0));
        locations.set(ObjId(4), ObjId(2));
        locations.set(ObjId(5), ObjId(2));
        locations.set(ObjId(6), ObjId(5));
        locations
    }

    fn assert(a: Vec<ObjId>, b: Vec<ObjId>) {
        let sa = a.into_iter().collect::<HashSet<_>>();
        let sb = b.into_iter().collect::<HashSet<_>>();

        assert_eq!(sa, sb);
    }
}
