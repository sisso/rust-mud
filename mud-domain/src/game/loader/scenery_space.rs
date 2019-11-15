use crate::game::container::Container;
use commons::{ObjId, V2};
use crate::game::room::RoomId;
use crate::game::domain::Dir;
use crate::game::labels::Label;
use crate::game::crafts::Craft;
use crate::game::surfaces::*;
use crate::game::planets::*;
use crate::game::pos::Pos;
use crate::game::surfaces_object::SurfaceObject;

type CraftId = ObjId;

pub fn load(container: &mut Container) {
    load_sector(container);
}

fn load_sector(container: &mut Container) {
    let sector_id = add_sector(container, "Sector 1");

    let planet1 = add_planet(container, "Dune", sector_id, V2::new(3.0, 4.0));
    let planet1_room1 = add_room(container, planet1, "Desert", "The grate deserts of dune!");
    let planet1_room2 = add_room(container, planet1, "Desert", "The grate deserts of dune!");
    add_portal(container,planet1_room1, planet1_room2, Dir::S);

    let planet2 = add_planet(container, "Planet 2", sector_id, V2::new(-2.0, 0.0));
    let planet2_room1 = add_room(container, planet2, "Vilalge", "The Chavez village");

    let (craft1, carft1_bridge) = add_craft(container, "Light Transport", sector_id, V2::new(0.0, 0.0));

    container.config.initial_room = carft1_bridge;
}

fn add_sector(container: &mut Container, label: &str) -> SurfaceId {
    let id = container.objects.create();
    container.labels.set(Label {
        id,
        label: label.to_string(),
        code: label.to_string(),
        desc: label.to_string(),
    });
    container.sectors.add(Surface::new(id));
    id
}

fn add_planet(container: &mut Container, label: &str, sector_id: SurfaceId, pos: V2) -> PlanetId {
    let id = container.objects.create();
    container.labels.set(Label {
        id,
        label: label.to_string(),
        code: label.to_string(),
        desc: label.to_string(),
    });
    container.locations.set(id, sector_id);
    container.planets.add(Planet::new(id));
    container.pos.set(Pos { id, pos });
    container.surface_objects.add(SurfaceObject::new(id));
    id
}

fn add_craft(container: &mut Container, label: &str, sector_id: SurfaceId, pos: V2) -> (CraftId, RoomId) {
    let id = container.objects.create();
    container.labels.set(Label::new(id, label));
    container.locations.set(id, sector_id);
    container.crafts.add(Craft::new(id));
    container.pos.set(Pos { id, pos });
    container.surface_objects.add(SurfaceObject::new(id));

    let bridge_id = add_craft_room(container, id, "Bridge", "Ship bridge");
    let cargo_id = add_craft_room(container, id, "Cargo", "Cargo hold");
    let airlock_id = add_craft_room(container, id, "Airlock", "Airlock");

    add_portal(container, bridge_id, cargo_id, Dir::S);
    add_portal(container, cargo_id, airlock_id, Dir::S);

    (id, bridge_id)
}

fn add_craft_room(container: &mut Container, craft_id: CraftId, label: &str, desc: &str) -> RoomId {
    let id = super::builder::add_room(container, label, desc);
    container.locations.set(id, craft_id);
    id
}

fn add_room(container: &mut Container, planet_id: PlanetId, label: &str, desc: &str) -> RoomId {
    let id = super::builder::add_room(container, label, desc);
    container.locations.set(id, planet_id);
    id
}

fn add_portal(container: &mut Container, room1_id: RoomId, room2_id: RoomId, dir: Dir) {
    container.rooms.add_portal(room1_id, room2_id, dir);
}

