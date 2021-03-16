use crate::errors::{AsResult, Error, Result};
use crate::game::comm;
use crate::game::container::Container;
use crate::game::inventory_service::{
    can_add_weight_by_prefab, update_all_current_inventory, update_inventory_weight,
};
use crate::game::loader::dto::StaticId;
use crate::game::loader::Loader;
use crate::game::mob::{MobId, EXTRACT_TIME};
use crate::utils::strinput::StrInput;
use commons::ObjId;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Extractable {
    pub id: ObjId,
    pub prefab_id: StaticId,
}

#[derive(Clone, Debug)]
pub struct Extractables {
    index: HashMap<ObjId, Extractable>,
}

// TODO: move mostly of this methods to a trait
impl Extractables {
    pub fn new() -> Self {
        Extractables {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, e: Extractable) -> Result<()> {
        if self.index.contains_key(&e.id) {
            return Err(Error::ConflictException);
        }
        self.index.insert(e.id, e);
        Ok(())
    }

    pub fn remove(&mut self, id: ObjId) -> Option<Extractable> {
        self.index.remove(&id)
    }

    pub fn get(&self, id: ObjId) -> Option<&Extractable> {
        self.index.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjId) -> Option<&mut Extractable> {
        self.index.get_mut(&id)
    }

    pub fn exist(&self, id: ObjId) -> bool {
        self.index.contains_key(&id)
    }

    pub fn list_ids<'a>(&'a self) -> impl Iterator<Item = &ObjId> + 'a {
        self.index.keys()
    }

    pub fn list<'a>(&'a self) -> impl Iterator<Item = &Extractable> + 'a {
        self.index.values()
    }
}

pub fn is_valid_extract(container: &Container, _mob_id: MobId, target_id: ObjId) -> bool {
    container.extractables.exist(target_id)
}

pub fn tick_extract(container: &mut Container, mob_id: MobId, target_id: ObjId) -> Result<bool> {
    if !is_valid_extract(container, mob_id, target_id) {
        let target_label = container.labels.get_label_f(target_id);

        container
            .outputs
            .private(mob_id, comm::extract_fail(target_label));

        return Ok(false);
    }

    let prefab_id = container.extractables.get(target_id).as_result()?.prefab_id;

    if !can_add_weight_by_prefab(container, mob_id, prefab_id) {
        let target_label = container.labels.get_label_f(target_id);
        let mob_label = container.labels.get_label_f(mob_id);
        let location_id = container.locations.get(mob_id).as_result()?;

        container.outputs.private(mob_id, comm::inventory_full());
        container.outputs.message(
            mob_id,
            location_id,
            comm::extract_stop(mob_label, target_label),
        );

        return Ok(false);
    }

    let mob = container.mobs.get_mut(mob_id).as_result()?;
    if mob.state.extract_calm_down.is_before(container.time.total) {
        mob.state.extract_calm_down.add(EXTRACT_TIME);

        Loader::spawn_at(container, prefab_id, mob_id);

        let target_label = container.labels.get_label_f(target_id);
        let mob_label = container.labels.get_label_f(mob_id);
        let prefab_label = container.loader.get_prefab_labelf(prefab_id);
        let location_id = container.locations.get(mob_id).as_result()?;

        let msg = comm::extract_success(mob_label, target_label, &prefab_label);
        container.outputs.message(mob_id, location_id, msg);

        update_inventory_weight(container, mob_id)?;

        Ok(true)
    } else {
        Ok(false)
    }
}
