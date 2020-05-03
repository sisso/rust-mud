use crate::game::labels::Labels;
use commons::save::{Snapshot, SnapshotSupport};
use commons::tree::Tree;
use commons::ObjId;
use logs::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type LocationId = ObjId;

#[derive(Clone, Debug)]
pub struct Locations {
    index: Tree<ObjId>,
}

impl Locations {
    pub fn new() -> Self {
        Locations { index: Tree::new() }
    }

    pub fn set(&mut self, obj_id: ObjId, location: ObjId) {
        info!("{:?} set location {:?}", obj_id, location);
        self.index.insert(obj_id, location);
    }

    pub fn remove(&mut self, obj_id: ObjId) {
        info!("{:?} remove location", obj_id);
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
}

pub fn search_at(
    labels: &Labels,
    locations: &Locations,
    location_id: LocationId,
    input: &str,
) -> Vec<ObjId> {
    let candidates = locations.list_at(location_id).collect::<Vec<_>>();
    labels.search_codes(&candidates, input)
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

impl SnapshotSupport for Locations {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        use serde_json::json;

        for (id, parent_id) in self.index.list_all() {
            if id.is_dynamic() {
                continue;
            }

            let value = json!(parent_id);
            snapshot.add(id.as_u32(), "location", value);
        }
    }
}
