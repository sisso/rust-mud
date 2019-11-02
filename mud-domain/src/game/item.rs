use commons::*;
use std::collections::HashMap;
use crate::game::Ctx;
use super::comm;
use super::container::Container;
use super::domain::NextId;
use super::mob::MobId;
use super::room::RoomId;
use logs::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemId(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemKind(pub u32);

pub const ITEM_KIND_UNDEFINED: ItemKind = ItemKind(0);
pub const ITEM_KIND_GOLD: ItemKind = ItemKind(1);
pub const ITEM_KIND_BODY: ItemKind = ItemKind(2);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ItemLocation {
    Limbo,
    Mob { mob_id: MobId },
    Room { room_id: RoomId },
    Item { item_id: ItemId },
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
    pub label: String,
    pub decay: Option<TotalTime>,
    pub amount: u32,
    pub item_def_id: Option<ItemPrefabId>,
    pub weapon: Option<Weapon>,
    pub armor: Option<Armor>,
}

impl Item {
    pub fn new(id: ItemId, typ: ItemKind, label: String) -> Self {
        Item {
            id,
            kind: typ,
            label,
            decay: None,
            amount: 1,
            item_def_id: None,
            weapon: None,
            armor: None
        }
    }

    pub fn can_equip(&self) -> bool {
        self.weapon.is_some() || self.armor.is_some()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemPrefabId(pub u32);

#[derive(Debug, Clone)]
pub struct ItemPrefab {
    pub id: ItemPrefabId,
    pub kind: ItemKind,
    pub label: String,
    pub amount: u32,
    pub weapon: Option<Weapon>,
    pub armor: Option<Armor>,
}

impl ItemPrefab {
    pub fn build(id: ItemPrefabId, label: String) -> ItemPrefabBuilder {
        let prefab = ItemPrefab { id, kind: ITEM_KIND_UNDEFINED, label, amount: 1, weapon: None, armor: None };

        ItemPrefabBuilder { prefab }
    }
}

#[derive(Debug, Clone)]
pub struct ItemPrefabBuilder {
    pub prefab: ItemPrefab
}

impl ItemPrefabBuilder {
    pub fn with_weapon(mut self, weapon: Weapon) -> Self {
        self.prefab.weapon = Some(weapon);
        self
    }

    pub fn with_armor(mut self, armor: Armor) -> Self {
        self.prefab.armor = Some(armor);
        self
    }

    pub fn with_kind(mut self, kind: ItemKind) -> Self {
        self.prefab.kind = kind;
        self
    }

    pub fn with_amount(mut self, amount: u32) -> Self {
        self.prefab.amount = amount;
        self
    }

    pub fn build(self) -> ItemPrefab {
        self.prefab
    }
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub damage_min: u32,
    pub damage_max: u32,
    pub reload: DeltaTime,
}

#[derive(Debug, Clone)]
pub struct Armor {
    pub rd: u32,
    pub dp: i32,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    location: ItemLocation,
    list: Vec<ItemId>,
    equip: Vec<ItemId>,
}

impl Inventory {
    pub fn new(location: ItemLocation) -> Self {
        Inventory {
            location: location,
            list: vec![],
            equip: vec![]
        }
    }

    pub fn add(&mut self, item_id: ItemId) {
        self.list.push(item_id);
    }

    pub fn remove(&mut self, item_id: &ItemId) {
        self.list.retain(|id| id != item_id);
    }
}

pub struct ItemRepository {
    next_item_id: NextId,
    next_item_def_id: NextId,
    index: HashMap<ItemId, Item>,
    inventory: HashMap<ItemLocation, Inventory>,
    prefab_index: HashMap<ItemPrefabId, ItemPrefab>,
    item_location: HashMap<ItemId, ItemLocation>,
}

impl ItemRepository {
    pub fn new() -> Self {
        ItemRepository {
            next_item_id: NextId::new(),
            next_item_def_id: NextId::new(),
            index: HashMap::new(),
            inventory: HashMap::new(),
            prefab_index: HashMap::new(),
            item_location: HashMap::new(),
        }
    }

    pub fn next_item_id(&mut self) -> ItemId {
        ItemId(self.next_item_id.next())
    }

    pub fn get(&self, item_id: ItemId) -> &Item {
        self.index.get(&item_id).unwrap()
    }

    pub fn set_location(&mut self, item_id: ItemId, location: ItemLocation) {
        let inventory = self.get_inventory_mut(location);
        inventory.add(item_id);

        self.item_location.insert(item_id, location);
    }

    pub fn move_all(&mut self, from: ItemLocation, to: ItemLocation) {
        debug!("itemrepostitory - move_all {:?} to {:?}", from, to);
        let from_inventory = self.inventory.remove(&from);
        if from_inventory.is_none() {
            return;
        }

        let from_inventory = from_inventory.unwrap();
        for item_id in from_inventory.list.iter() {
            debug!("itemrepostitory - set {:?} to {:?}", item_id, to);
            self.item_location.insert(*item_id, to);
        }

        let to_inventory = self.get_inventory_mut(to);
        for item_id in from_inventory.list {
            debug!("itemrepostitory - add {:?} to {:?}", item_id, to_inventory);
            to_inventory.add(item_id);
        }
    }

    pub fn move_item(&mut self, item_id: ItemId, location: ItemLocation) {
        self.remove_location(item_id);
        self.add_location(&item_id, location);
    }

    pub fn move_items_from_mob_to_item(&mut self, mob_id: MobId, item_id: ItemId) {
        self.move_all(ItemLocation::Mob { mob_id: mob_id }, ItemLocation::Item { item_id: item_id });
    }

    pub fn get_equiped(&self, location: ItemLocation) -> Vec<ItemId> {
        match self.inventory.get(&location) {
            Some(inventory) => inventory.equip.clone(),
            None => vec![],
        }
    }

    pub fn get_inventory_list(&self, location: ItemLocation) -> Vec<&Item> {
        match self.inventory.get(&location) {
            Some(inventory) => {
                inventory
                    .list
                    .iter()
                    .map(|item_id| {
                        self.get(*item_id)
                    })
                    .collect()
            }
            None => vec![],
        }
    }

    pub fn search_inventory(&self, location: ItemLocation, item_label: &str) -> Option<&Item> {
        let inventory = self.get_inventory_list(location);
        let item: Option<&Item> = inventory.iter().find(|item| item.label.eq_ignore_ascii_case(item_label)).map(|i| *i);
        item
    }

    pub fn add(&mut self, item: Item, location: ItemLocation) {
        let item_id = item.id;

        if self.index.contains_key(&item_id) {
            panic!()
        }

        // update index
        self.index.insert(item_id, item);

        // update inventory
        let inventory = self.get_inventory_mut(location);
        inventory.add(item_id);

        // update location
        self.item_location.insert(item_id, location);
    }

    pub fn add_prefab(&mut self, item_def: ItemPrefab) {
        if self.prefab_index.contains_key(&item_def.id) {
            panic!(format!("item prefab {:?} already exists, failed ot insert {:?}", item_def.id, item_def));
        }
        self.prefab_index.insert(item_def.id, item_def);
    }

    pub fn remove(&mut self, item_id: ItemId) {
        let item = self.index.remove(&item_id).unwrap();
        let location = self.item_location.remove(&item_id);
        if let Some(location) = location {
            let inventory = self.get_inventory_mut(location);
            inventory.remove(&item_id);
        }

        // TODO: remove recursive all items in inventory
    }

    pub fn list(&self) -> Vec<ItemId> {
        self.index.keys().map(|i| *i).collect()
    }

    pub fn get_prefab(&self, item_prefab_id: &ItemPrefabId) -> &ItemPrefab {
        self.prefab_index.get(item_prefab_id).unwrap()
    }

    pub fn search(&self, room_id: &RoomId, name: &String) -> Vec<&Item> {
        match self.inventory.get(&ItemLocation::Room { room_id: *room_id }) {
            Some(inventory) => {
                inventory.list.iter().filter_map(|item_id| {
                    let item = self.get(*item_id);
                    if item.label.eq_ignore_ascii_case(name) {
                        Some(item)
                    } else {
                        None
                    }
                }).collect()
            }
            _ => {
                vec![]
            }
        }
    }

    pub fn get_location(&self, item_id: ItemId) -> &ItemLocation {
        self.item_location.get(&item_id).unwrap()
    }

    pub fn get_inventory(&self, location: ItemLocation) -> Option<&Inventory> {
        self.inventory.get(&location)
    }

    pub fn equip(&mut self, location: ItemLocation, item_id: ItemId) -> Result<(),()> {
        let item = self.index.get(&item_id).unwrap();
        let is_weapon = item.weapon.is_some();
        let is_armor = item.armor.is_some();

        if !is_weapon && !is_armor {
            debug!("itemrepostitory - {:?} try to equip invalid item {:?}", location, item_id);
            return Err(());
        }

        let mut inventory = self.get_inventory_mut(location).clone();

        // remove equipments of same type
        inventory.equip.retain(|item_id| {
            let item = self.index.get(&item_id).unwrap();
            let remove = is_weapon && item.weapon.is_some() || is_armor && item.armor.is_some();
            if remove {
                debug!("itemrepostitory - {:?} unequip {:?}", location, item_id);
            }
            !remove
        });

        // add new item
        inventory.equip.push(item_id);
        self.inventory.insert(location, inventory);

        debug!("itemrepostitory - {:?} equip {:?}", location, item_id);

        Ok(())
    }

    pub fn strip(&mut self, item_id: ItemId) -> Result<(),()> {
        Err(())
    }

    fn get_inventory_mut(&mut self, location: ItemLocation) -> &mut Inventory {
        self.inventory.entry(location).or_insert(Inventory::new(location.clone()))
    }

    fn remove_location(&mut self, item_id: ItemId) {
        let location = self.get_location(item_id).clone();
        let inventory = self.get_inventory_mut(location);
        inventory.remove(&item_id);

        self.item_location.remove(&item_id);

        debug!("itemrepostitory - remove_location {:?}", item_id);
    }

    fn add_location(&mut self, item_id: &ItemId, location: ItemLocation) {
        let inventory = self.get_inventory_mut(location);
        inventory.add(*item_id);

        self.item_location.insert(*item_id, location);

        debug!("itemrepostitory - add_location {:?} {:?}", item_id, location);
    }

    pub fn instantiate_item(&mut self, item_prefab_id: ItemPrefabId, location: ItemLocation) -> ItemId {
        let item_id = self.next_item_id();
        let prefab = self.get_prefab(&item_prefab_id);

        let mut item = Item::new(
            item_id,
            prefab.kind,
            prefab.label.clone()
        );

        item.amount = prefab.amount;
        item.item_def_id = Some(item_prefab_id);
        item.weapon = prefab.weapon.clone();
        item.armor = prefab.armor.clone();

        self.add(item, location);

        item_id
    }

//    pub fn save(&self, save: &mut dyn Save) {
//        use serde_json::json;
//
//        for (id, obj) in self.index.iter() {
//            let obj_json = json!({
//                "kind": obj.kind.0,
//                "label": obj.label,
//                "decay": obj.decay.map(|i| i.0),
//                "amount": obj.amount,
//                "definition_id": obj.item_def_id.map(|i| i.0)
//            });
//
//            save.add(id.0, "item", obj_json);
//        }
//
//        for (id, (location, inventory)) in self.inventory.iter().enumerate() {
//            let location_json = match location {
//                ItemLocation::Limbo => json!({"kind": "limbo"}),
//                ItemLocation::Mob { mob_id } => json!({"kind": "mob", "mob_id": mob_id.0 }),
//                ItemLocation::Room { room_id } => json!({"kind": "room", "room_id": room_id.0 }),
//                ItemLocation::Item { item_id } => json!({"kind": "item", "item_id": item_id.0 }),
//            };
//
//            let obj_json = json!({
//                "location": location_json,
//                "items": inventory.list.iter().map(|i| i.0).collect::<Vec<u32>>()
//            });
//
//            save.add(id as u32, "inventory", obj_json);
//        }
//    }
}

pub fn run_tick(ctx: &mut Ctx) {
    ctx.container
        .items
        .list()
        .into_iter()
        .for_each(|item_id| {
            let item = ctx.container.items.get(item_id);

            if let Some(decay) = item.decay {
                let location = ctx.container.items.get_location(item.id);

                match location {
                    ItemLocation::Room { room_id } if decay.is_before(ctx.container.time.total) => {
                        let msg = comm::item_body_disappears(item);
                        ctx.outputs.room_all(*room_id, msg);
                        ctx.container.items.remove(item_id);
                    }
                    _ => {}
                }
            }
        });
}


#[derive(Debug)]
pub enum ParseItemError {
    ItemNotProvided,
    ItemNotFound { label: String },
}

pub fn parser_item(container: &mut Container, item_location: ItemLocation, args: Vec<String>) -> Result<ItemId, ParseItemError> {
    let item_label = match args.get(1) {
        Some(str) => str,
        None => return Err(ParseItemError::ItemNotProvided),
    };

    match container.items.search_inventory(item_location, item_label) {
        Some(item) => Ok(item.id),
        None => Err(ParseItemError::ItemNotFound { label: item_label.to_string() }),
    }
}

