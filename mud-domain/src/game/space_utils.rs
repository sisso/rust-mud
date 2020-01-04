use crate::errors::{AsResult, Error, Result};
use crate::game::astro_bodies::AstroBody;
use crate::game::comm::{
    ShowSectorTreeBody, ShowSectorTreeBodyKind, ShowStarmapDescKind, SurfaceDesc,
};
use crate::game::container::Container;
use crate::game::crafts::ShipId;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::{comm, Outputs};
use commons::{ObjId, PlayerId, MIN_DISTANCE, V2};

pub fn find_surface_target(
    container: &mut Container,
    craft_location: ObjId,
    label: &str,
) -> Result<ObjId> {
    let candidates = container
        .locations
        .list_at(craft_location)
        .collect::<Vec<_>>();

    let founds = container.labels.search_codes(&candidates, label);
    founds.first().cloned().ok_or(Error::NotFoundFailure)
}

pub fn get_objects_in_surface(
    container: &Container,
    craft_id: ObjId,
    craft_location: ObjId,
) -> Vec<SurfaceDesc> {
    let objects = container
        .locations
        .list_at(craft_location)
        .flat_map(|id| {
            let label = container.labels.get_label_f(id);
            let pos = container.pos.get_pos(id);
            let is_craft = container.ship.exists(id);
            let is_planet = container.astro_bodies.exists(id);

            match pos {
                Some(pos) if is_craft => Some(SurfaceDesc {
                    kind: ShowStarmapDescKind::Craft,
                    pos: pos,
                    me: id == craft_id,
                    label: label.to_string(),
                }),
                Some(pos) if is_planet => Some(SurfaceDesc {
                    kind: ShowStarmapDescKind::Planet,
                    pos: pos,
                    me: false,
                    label: label.to_string(),
                }),
                _ => None,
            }
        })
        .collect();

    objects
}

pub fn search_near_landing_sites(container: &Container, craft_id: ObjId) -> Vec<ObjId> {
    container
        .locations
        .get(craft_id)
        .and_then(|location_id| {
            container
                .pos
                .get_pos(craft_id)
                .map(|pos| (location_id, pos))
        })
        .map(|(location_id, pos)| {
            container
                .locations
                .list_at(location_id)
                .filter(|&obj_id| {
                    if !container.astro_bodies.exists(obj_id) {
                        return false;
                    }

                    let is_near = container
                        .pos
                        .get_pos(obj_id)
                        .map(|planet_pos| {
                            let distance = planet_pos.distance(pos);
                            distance <= MIN_DISTANCE
                        })
                        .unwrap_or(false);

                    is_near
                })
                .flat_map(|planet_id| {
                    container.locations.list_at(planet_id).filter(|&id| {
                        container
                            .rooms
                            .get(id)
                            .map(|room| room.can_exit)
                            .unwrap_or(false)
                    })
                })
                .collect()
        })
        .unwrap_or(vec![])
}

pub fn get_craft_and_sector(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<(ShipId, ObjId)> {
    let craft_id = match get_craft(container, mob_id) {
        Some(craft_id) => craft_id,
        None => {
            outputs.private(mob_id, comm::space_not_in_craft());
            return Err(Error::NotFoundFailure);
        }
    };

    let craft_location = container.locations.get(craft_id).as_result()?;
    if !container.sectors.exists(craft_location) {
        outputs.private(mob_id, comm::space_not_in_craft());
        return Err(Error::NotFoundFailure);
    }

    return Ok((craft_id, craft_location));
}

pub fn get_craft(container: &Container, mob_id: MobId) -> Option<ShipId> {
    let room_id = container.locations.get(mob_id)?;
    if !container.rooms.exists(room_id) {
        return None;
    }

    let craft_id = container.locations.get(room_id)?;
    if !container.ship.exists(craft_id) {
        return None;
    }

    Some(craft_id)
}

pub fn find_ships_at(container: &Container, location_id: LocationId) -> Vec<ShipId> {
    container
        .locations
        .list_at(location_id)
        .filter(|&id| container.ship.exists(id))
        .collect()
}

pub fn find_children_rooms_with_can_exit(
    container: &Container,
    location_id: LocationId,
) -> Vec<LocationId> {
    container
        .locations
        .list_at(location_id)
        .flat_map(|id| container.rooms.get(id))
        .filter(|room| room.can_exit)
        .map(|room| room.id)
        .collect()
}

pub fn find_astro_bodies(container: &Container, sector_id: ObjId) -> Vec<ShowSectorTreeBody> {
    container
        .locations
        .list_deep_at(sector_id)
        .into_iter()
        .flat_map(|id| to_showsectortreebody(container, id))
        .collect()
}

fn to_showsectortreebody(container: &Container, obj_id: ObjId) -> Option<ShowSectorTreeBody> {
    match (
        container.astro_bodies.get(obj_id),
        container.ship.get(obj_id),
    ) {
        (Some(body), None) => {
            let body = ShowSectorTreeBody {
                id: obj_id,
                label: container.labels.get_label_f(obj_id),
                orbit_id: body.orbit_id,
                kind: ShowSectorTreeBodyKind::Unknown,
            };

            Some(body)
        }
        (_, Some(_ship)) => {
            let body = ShowSectorTreeBody {
                id: obj_id,
                label: container.labels.get_label_f(obj_id),
                // FIXME: implement
                orbit_id: None,
                kind: ShowSectorTreeBodyKind::Ship,
            };

            Some(body)
        }
        _ => None,
    }
}
