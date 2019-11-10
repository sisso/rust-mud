use crate::game::container::Container;
use commons::ObjId;
use crate::game::room::RoomId;
use crate::game::domain::Dir;
use crate::game::labels::Label;
use crate::game::crafts::Craft;

type SectorId = ObjId;
type PlanetId = ObjId;
type CraftId = ObjId;

pub fn load(container: &mut Container) {
    load_sector(container);
}

fn load_sector(container: &mut Container) {
    let sector_id = add_sector(container, "Sector 1");

    let planet1 = add_planet(container, "Dune", sector_id);
    let planet1_room1 = add_room(container, planet1, "Desert", "The grate deserts of dune!");
    let planet1_room2 = add_room(container, planet1, "Desert", "The grate deserts of dune!");
    add_portal(container,planet1_room1, planet1_room2, Dir::S);

    let planet2 = add_planet(container, "Planet 2", sector_id);
    let planet2_room1 = add_room(container, planet2, "Vilalge", "The Chavez village");

    let craft1 = add_craft(container, "Light Transport");

    container.config.initial_room = planet1_room1;
}

fn add_sector(container: &mut Container, label: &str) -> SectorId {
    let id = container.objects.create();
    container.labels.set(Label {
        id,
        label: label.to_string(),
        code: label.to_string(),
        desc: label.to_string(),
    });
    id
}

fn add_planet(container: &mut Container, label: &str, sector_id: SectorId) -> PlanetId {
    let id = container.objects.create();
    container.labels.set(Label {
        id,
        label: label.to_string(),
        code: label.to_string(),
        desc: label.to_string(),
    });
    container.locations.set(id, sector_id);
    id
}

fn add_craft(container: &mut Container, label: &str) -> CraftId {
    let id = container.objects.create();
    container.labels.set(Label::new(id, label));
    container.locations.set(id, sector_id);
    container.crafts.add(id, Craft::new(id));
    id
}

fn add_room(container: &mut Container, planet_id: PlanetId, label: &str, desc: &str) -> RoomId {
    let id = super::builder::add_room(container, label, desc);
    container.locations.set(id, planet_id);
    id
}

fn add_portal(container: &mut Container, room1_id: RoomId, room2_id: RoomId, dir: Dir) {
    let room1 = container.rooms.get_mut(room1_id).unwrap();
    room1.exits.push((dir, room2_id));

    let room2 = container.rooms.get_mut(room2_id).unwrap();
    room2.exits.push((dir.inv(), room1_id));
}

