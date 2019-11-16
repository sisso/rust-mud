use std::path::Path;

use crate::game::container::Container;
use crate::game::item::*;
use crate::game::mob::*;
use crate::game::domain::*;
use crate::game::room::*;
use crate::game::spawn::*;
use commons::*;
use commons::save::Load;
use crate::game::labels::Label;
use crate::game::builder;

pub mod scenery_space;

const ID_ROOM_INIT: RoomId = ObjId(0);
const ID_ROOM_BAR: RoomId = ObjId(1);
const ID_ROOM_FLOREST: RoomId = ObjId(2);
const ID_PREFAB_MOB_PLAYER: MobPrefabId = ObjId(3);
const ID_PREFAB_MOB_DRUNK: MobPrefabId = ObjId(4);
const ID_ITEM_DEF_COINS_2: ItemPrefabId = ObjId(5);
const ID_ITEM_DEF_SWORD: ItemPrefabId = ObjId(6);
const ID_ITEM_DEF_ARMOR: ItemPrefabId = ObjId(7);
const ID_ITEM_DEF_CHEST: ItemPrefabId = ObjId(8);

pub fn load_ids(container: &mut Container) {
    for id in vec![
        ID_ROOM_INIT,
        ID_ROOM_BAR,
        ID_ROOM_FLOREST,
        ID_PREFAB_MOB_PLAYER,
        ID_PREFAB_MOB_DRUNK,
        ID_ITEM_DEF_COINS_2,
        ID_ITEM_DEF_SWORD,
        ID_ITEM_DEF_ARMOR,
    ] {
        container.objects.insert(id);
    }
}

fn load_items_prefabs(container: &mut Container) {
    container.items.add_prefab(
        ItemPrefab::build(ID_ITEM_DEF_COINS_2, "coins".to_string())
            .with_kind(ITEM_KIND_GOLD)
            .with_amount(2)
            .build()
    );

    container.items.add_prefab(
        ItemPrefab::build(ID_ITEM_DEF_SWORD, "sword".to_string())
            .with_weapon(Weapon {
                damage_min: 2,
                damage_max: 4,
                reload: DeltaTime(1.0)
            })
            .build()
    );

    container.items.add_prefab(
        ItemPrefab::build(ID_ITEM_DEF_ARMOR, "armor".to_string())
            .with_armor(Armor {
                rd: 2,
                dp: 1
            })
            .build()
    );

    container.items.add_prefab(
        ItemPrefab::build(ID_ITEM_DEF_CHEST, "chest".to_string())
            .with_inventory()
            .with_stuck()
            .build()
    );
}

fn load_mobs_prefabs(container: &mut Container) {
    container.mobs.add_prefab(MobPrefab {
        id: ID_PREFAB_MOB_PLAYER,
        label: "Avatar".to_string(),
        attributes: Attributes {
            attack: 12,
            defense: 12,
            damage: Damage { min: 2, max: 4 },
            pv: Pv { current: 10, max: 10, heal_rate: DeltaTime(1.0) },
            attack_calm_down: DeltaTime(1.0)
        },
        inventory: vec![],
    });

    container.mobs.add_prefab(MobPrefab {
        id: ID_PREFAB_MOB_DRUNK,
        label: "Drunk".to_string(),
        attributes: Attributes {
            attack: 8,
            defense: 8,
            damage: Damage { min: 1, max: 2 },
            pv: Pv { current: 8, max: 8, heal_rate: DeltaTime(1.0) },
            attack_calm_down: DeltaTime(1.0),
        },
        inventory: vec![
            ID_ITEM_DEF_COINS_2
        ],
    });
}

fn load_rooms(container: &mut Container) {
    let room1 = Room {
        id: ID_ROOM_INIT,
        exits: vec![(Dir::S, ID_ROOM_BAR)],
        is_airlock: false
    };
    container.rooms.add(room1);
    container.labels.set(Label::new_desc(ID_ROOM_INIT, "Main Room", "Where new characters born."));

    let room2 = Room {
        id: ID_ROOM_BAR,
        exits: vec![(Dir::N, ID_ROOM_INIT), (Dir::S, ID_ROOM_FLOREST)],
        is_airlock: false
    };
    container.rooms.add(room2);
    container.labels.set(Label::new_desc(ID_ROOM_BAR, "Bar", "A dirty bar where people come to relax"));

    let room3 = Room {
        id: ID_ROOM_FLOREST,
        exits: vec![(Dir::N, ID_ROOM_BAR)],
        is_airlock: false
    };
    container.labels.set(Label::new_desc(ID_ROOM_FLOREST, "Dark forest", "A dark forest where you think you can die"));
    container.rooms.add(room3);
}

fn load_spawns(container: &mut Container) {
    let spawn_id = container.objects.create();

    container.spawns.add(Spawn {
        id: spawn_id,
        room_id: ID_ROOM_BAR,
        max: 1,
        delay: SpawnDelay {
            min: DeltaTime(5.0),
            max: DeltaTime(20.0),
        },
        prefab_id: ID_PREFAB_MOB_DRUNK,
        next: Some(TotalTime(1.0)),
        mobs_id: vec![],
    });
}

fn create_item_at(container: &mut Container, item_prefab_id: ItemPrefabId, location_id: ObjId) -> ItemId {
    builder::add_item_from_prefab(container, item_prefab_id, location_id)
}

pub fn load_rooms_objects(container: &mut Container) {
    let chest_id = create_item_at(container, ID_ITEM_DEF_CHEST, ID_ROOM_FLOREST);
    create_item_at(container, ID_ITEM_DEF_ARMOR, chest_id);
    create_item_at(container, ID_ITEM_DEF_SWORD,chest_id);
}

pub fn load(container: &mut Container) {
    load_ids(container);
    load_items_prefabs(container);
    load_mobs_prefabs(container);
    load_rooms(container);
    load_rooms_objects(container);
    load_spawns(container);
}

pub mod hocon_loader;

#[derive(Debug)]
pub enum LoaderError {
    Unknown
}
impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unknown error when parsing a configuration file")
    }
}

impl std::error::Error for LoaderError {
    fn description(&self) -> &str {
        "unknown error when parsing a configuration file"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Loader {
    fn load(path: &Path) -> Result<Box<dyn Load>>;
}
