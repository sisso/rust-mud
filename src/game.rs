mod actions;
mod comm;
mod controller;
mod domain;
mod players;
mod spawn;
mod view_main;
mod view_login;

use crate::server;

use controller::*;
use domain::*;
use spawn::*;

fn load_mobs_prefabs(container: &mut Container) {
    let id_0_drunk = MobPrefab {
        id: MobPrefabId(0),
        label: "Drunk".to_string(),
    };

    container.add_mob_prefab(id_0_drunk);
}

fn load_rooms(container: &mut Container) {
    let room1 = Room {
        id: 0,
        label: "Main Room".to_string(),
        desc: "Main room where people born".to_string(),
        exits: vec![(Dir::S, 1)],
    };

    let room2 = Room {
        id: 1,
        label: "Bar".to_string(),
        desc: "Where we relief our duties".to_string(),
        exits: vec![(Dir::N, 0)],
    };

    container.add_room(room1);
    container.add_room(room2);
}

fn load_spawns(container: &mut Container) {
    container.add_spawn(Spawn {
        id: SpawnId(0),
        room_id: RoomId(1),
        max: 4,
        delay: SpawnDelay {
            min: Seconds(5.0),
            max: Seconds(20.0),
        },
        prefab_id: MobPrefabId(0),
        next: None,
        mobs_id: vec![]
    });
}

fn load(container: &mut Container) {
    load_mobs_prefabs(container);
    load_rooms(container);
    load_spawns(container);
}

pub fn run() {
    let mut container = Container::new();
    load(&mut container);

    let mut controller = GameController::new(container);

    let mut server = server::Server::new();
    server.start();

    let mut pending_outputs: Vec<server::Output> = vec![];

    loop {
        let result = server.run(pending_outputs);

        let params = controller::GameControllerContext {
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs
        };

        pending_outputs = controller.handle(params);

        std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
