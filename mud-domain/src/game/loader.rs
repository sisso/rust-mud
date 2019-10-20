use std::path::Path;

use crate::game::container::Container;
use crate::game::item::*;
use crate::game::mob::*;
use crate::game::domain::*;
use crate::game::room::*;
use crate::game::spawn::*;
use crate::utils::*;
use crate::utils::save::Load;


const MOB_PLAYER: MobPrefabId = MobPrefabId(0);
const MOB_DRUNK: MobPrefabId = MobPrefabId(1);

const ITEM_DEF_COINS_2: ItemPrefabId = ItemPrefabId(0);
const ITEM_DEF_SWORD: ItemPrefabId = ItemPrefabId(1);
const ITEM_DEF_ARMOR: ItemPrefabId = ItemPrefabId(2);

const ROOM_ID_FLOREST: RoomId = RoomId(2);

fn load_items_prefabs(container: &mut Container) {
    container.items.add_prefab(
        ItemPrefab::build(ITEM_DEF_COINS_2, "coins".to_string())
            .with_kind(ITEM_KIND_GOLD)
            .with_amount(2)
            .build()
    );

    container.items.add_prefab(
        ItemPrefab::build(ITEM_DEF_SWORD, "sword".to_string())
            .with_weapon(Weapon {
                damage_min: 2,
                damage_max: 4,
                reload: Second::one()
            })
            .build()
    );

    container.items.add_prefab(
        ItemPrefab::build(ITEM_DEF_ARMOR, "armor".to_string())
            .with_armor(Armor {
                rd: 2,
                dp: 1
            })
            .build()
    );
}

fn load_mobs_prefabs(container: &mut Container) {
    container.mobs.add_prefab(MobPrefab {
        id: MOB_PLAYER,
        label: "Avatar".to_string(),
        attributes: Attributes {
            attack: 12,
            defense: 12,
            damage: Damage { min: 2, max: 4 },
            pv: Pv { current: 10, max: 10, heal_rate: Second(1.0) },
            attack_calm_down: Second(1.0)
        },
        inventory: vec![],
    });

    container.mobs.add_prefab(MobPrefab {
        id: MOB_DRUNK,
        label: "Drunk".to_string(),
        attributes: Attributes {
            attack: 8,
            defense: 8,
            damage: Damage { min: 1, max: 2 },
            pv: Pv { current: 8, max: 8, heal_rate: Second(1.0) },
            attack_calm_down: Second(1.0),
        },
        inventory: vec![
            ITEM_DEF_COINS_2
        ],
    });
}

fn load_rooms(container: &mut Container) {
    let room_id_bar = RoomId(1);

    let room1 = Room {
        id: INITIAL_ROOM_ID,
        label: "Main Room".to_string(),
        desc: "Main room where people born".to_string(),
        exits: vec![(Dir::S, room_id_bar)],
    };

    let room2 = Room {
        id: room_id_bar,
        label: "Bar".to_string(),
        desc: "Where we relief our duties".to_string(),
        exits: vec![(Dir::N, INITIAL_ROOM_ID), (Dir::S, ROOM_ID_FLOREST)],
    };

    let room3 = Room {
        id: ROOM_ID_FLOREST,
        label: "Florest".to_string(),
        desc: "A deep, ugly and dark florest.".to_string(),
        exits: vec![(Dir::N, room_id_bar)],
    };

    container.rooms.add(room1);
    container.rooms.add(room2);
    container.rooms.add(room3);
}

fn load_spawns(container: &mut Container) {
    container.add_spawn(Spawn {
        id: SpawnId(0),
        room_id: RoomId(1),
        max: 1,
        delay: SpawnDelay {
            min: Second(5.0),
            max: Second(20.0),
        },
        prefab_id: MOB_DRUNK,
        next: Some(Second(1.0)),
        mobs_id: vec![],
    });
}

pub fn load_rooms_objects(container: &mut Container) {
    container.items.instantiate_item(ITEM_DEF_ARMOR, ItemLocation::Room { room_id: ROOM_ID_FLOREST });
    container.items.instantiate_item(ITEM_DEF_SWORD, ItemLocation::Room { room_id: ROOM_ID_FLOREST });
}

pub fn load(container: &mut Container) {
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
