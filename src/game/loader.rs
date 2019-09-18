use crate::game::container::Container;
use crate::game::item::*;
use crate::game::{ITEM_DEF_COINS_2, MOB_PLAYER, MOB_DRUNK};
use crate::game::mob::*;
use crate::game::domain::*;
use crate::game::room::*;
use crate::game::spawn::*;
use crate::utils::*;

fn load_items_prefabs(container: &mut Container) {
    container.items.add_prefab(ItemPrefab {
        id: ITEM_DEF_COINS_2,
        typ: ITEM_TYPE_GOLD,
        amount: 2,
        label: "coins".to_string(),
    });
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
    let room_id_florest = RoomId(2);

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
        exits: vec![(Dir::N, INITIAL_ROOM_ID), (Dir::S, room_id_florest)],
    };

    let room3 = Room {
        id: room_id_florest,
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

pub fn load(container: &mut Container) {
    load_items_prefabs(container);
    load_mobs_prefabs(container);
    load_rooms(container);
    load_spawns(container);
}
