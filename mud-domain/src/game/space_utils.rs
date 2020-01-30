use crate::errors::{AsResult, Error, Result};
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::comm::{ShowSectorTreeBody, ShowSectorTreeBodyKind, ShowStarmapDescKind, SurfaceDesc, ShowSectorTreeBodyOrbit};
use crate::game::container::Container;
use crate::game::ships::ShipId;
use crate::game::location::LocationId;
use crate::game::mob::MobId;
use crate::game::{comm, Outputs};
use commons::{ObjId, PlayerId, MIN_DISTANCE, V2};

#[deprecated]
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

#[deprecated]
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
            let is_craft = container.ships.exists(id);
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

pub fn get_ship_and_sector(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<(ShipId, ObjId)> {
    let ship_id = match get_ship(container, mob_id) {
        Some(craft_id) => craft_id,
        None => {
            outputs.private(mob_id, comm::space_not_in_craft());
            return Err(Error::NotFoundFailure);
        }
    };

    let sector = container.locations.list_parents(ship_id)
        .into_iter()
        .find(|&obj_id| container.sectors.exists(obj_id));

    let sector_id = sector.ok_or_else(|| {
        outputs.private(mob_id, comm::space_not_in_craft());
        Error::NotFoundFailure
    })?;

    return Ok((ship_id, sector_id));
}

pub fn get_ship(container: &Container, mob_id: MobId) -> Option<ShipId> {
    let room_id = container.locations.get(mob_id)?;
    if !container.rooms.exists(room_id) {
        return None;
    }

    let craft_id = container.locations.get(room_id)?;
    if !container.ships.exists(craft_id) {
        return None;
    }

    Some(craft_id)
}

pub fn find_ships_at(container: &Container, location_id: LocationId) -> Vec<ShipId> {
    container
        .locations
        .list_at(location_id)
        .filter(|&id| container.ships.exists(id))
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

pub fn find_space_bodies(container: &Container, sector_id: ObjId) -> Vec<&AstroBody> {
    container
        .locations
        .list_deep_at(sector_id)
        .into_iter()
        .flat_map(|id| container.astro_bodies.get(id))
        .collect()
}

pub fn find_showsector_bodies(container: &Container, sector_id: ObjId, ship_id: Option<ObjId>) -> Vec<ShowSectorTreeBody> {
    find_space_bodies(container, sector_id)
        .into_iter()
        .map(|body| to_showsectortreebody(container, ship_id, body))
        .collect()
}

pub fn to_showsectortreebody<'a>(container: &'a Container, self_id: Option<ObjId>, body: &'a AstroBody) -> ShowSectorTreeBody<'a> {
    let obj_id = body.id;

    ShowSectorTreeBody {
        id: obj_id,
        label: container.labels.get_label_f(obj_id),
        orbit_id:  container.locations.get(obj_id).expect("Space objects must have a orbit"),
        orbit_distance: body.orbit_distance,
        kind: body.kind.into(),
        is_self: Some(obj_id) == self_id
    }
}
