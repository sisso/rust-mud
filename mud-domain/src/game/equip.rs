use std::collections::HashSet;
use std::collections::HashMap;
use commons::ObjId;
use crate::game::mob::MobId;
use crate::game::item::ItemId;

#[derive(Clone,Debug)]
pub struct Equip {
    pub obj_id: MobId,
    pub equipments: HashSet<ItemId>,
}

impl Equip {
    pub fn new(obj_id: MobId) -> Self {
        Equip { obj_id, equipments: HashSet::new() }
    }
}

/// What a object is using wielding or wearing
#[derive(Clone,Debug)]
pub struct Equips {
    index: HashMap<ObjId, Equip>,
}

impl Equips {
    pub fn new() -> Self {
        Equips {
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, mob_id: MobId, item_id: ItemId) {
        let equip = self.index.entry(mob_id).or_insert(Equip::new(mob_id));
        equip.equipments.insert(item_id);
    }

    pub fn strip(&mut self, mob_id: MobId, item_id: ItemId) -> Result<(),()> {
        match self.index.get_mut(&mob_id) {
            Some(equip) => {
                let removed = equip.equipments.remove(&item_id);
                if !removed {
                    return Err(());
                }
                Ok(())
            },
            None => Err(())
        }
    }

    pub fn get(&self, id: MobId) -> Result<HashSet<ItemId>, ()> {
        self.index.get(&id)
            .map(|equip| equip.equipments.clone())
            .ok_or(())
    }

    pub fn remove(&mut self, id: ObjId) -> Result<(),()> {
        self.index.iter_mut().for_each(|(_, equip)| {
           equip.equipments.remove(&id);
        });

        Ok(())
    }
}

