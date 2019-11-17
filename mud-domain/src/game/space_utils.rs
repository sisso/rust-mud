use commons::{ObjId, PlayerId, MIN_DISTANCE, AsResult};
use crate::game::{comm, Outputs};
use crate::game::comm::{ShowStarmapDescKind, SurfaceDesc};
use crate::game::container::Container;
use crate::game::crafts::CraftId;
use crate::game::mob::MobId;
use crate::game::location::LocationId;

pub fn find_surface_target(container: &mut Container, craft_location: ObjId, label: &str) -> Result<ObjId,()> {
    let candidates = container.locations.list_at(craft_location).collect::<Vec<_>>();
    let founds = container.labels.search_codes(&candidates, label);
    founds.first().cloned().ok_or(())
}

pub fn get_objects_in_surface(container: &Container, craft_id: ObjId, craft_location: ObjId) -> Vec<SurfaceDesc> {
    let objects = container.locations.list_at(craft_location).flat_map(|id| {
        let label = container.labels.get_label_f(id);
        let pos = container.pos.get_pos(id);
        let is_craft = container.crafts.exists(id);
        let is_planet = container.planets.exists(id);

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
            _ => None
        }
    }).collect();

    objects
}

pub fn search_near_landing_sites(container: &Container, craft_id: ObjId) -> Vec<ObjId> {
    container.locations.get(craft_id)
        .and_then(|location_id| {
            container.pos.get_pos(craft_id).map(|pos| {
                (location_id, pos)
            })
        })
        .map(|(location_id, pos)|{
            container.locations.list_at(location_id)
                .filter(|&obj_id| {
                    if !container.planets.exists(obj_id) {
                        return false;
                    }

                    let is_near = container.pos.get_pos(obj_id).map(|planet_pos| {
                        let distance = planet_pos.distance(pos);
                        distance <= MIN_DISTANCE
                    }).unwrap_or(false);

                    is_near
                })
                .flat_map(|planet_id| {
                    container.locations.list_at(planet_id)
                        .filter(|&id| {
                            container.rooms
                                .get(id)
                                .map(|room| room.is_airlock)
                                .unwrap_or(false)
                        })
                }).collect()
        }).unwrap_or(vec![])
}

pub fn get_craft_and_location(container: &Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(CraftId, ObjId),()> {
    let craft_id = match get_craft(container, mob_id) {
        Ok(craft_id) => craft_id,
        Err(()) => {
            outputs.private(player_id, comm::space_not_in_craft());
            return Err(());
        }
    };

    let craft_location = container.locations.get(craft_id).as_result()?;
    if !container.sectors.exists(craft_location) {
        outputs.private(player_id, comm::space_not_in_craft());
        return Err(());
    }

    return Ok((craft_id, craft_location));
}

pub fn get_craft(container: &Container, mob_id: MobId) -> Result<CraftId, ()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    if !container.rooms.exists(room_id) {
        return Err(());
    }

    let craft_id = container.locations.get(room_id).as_result()?;
    if !container.crafts.exists(craft_id) {
        return Err(());
    }

    Ok(craft_id)
}

pub fn get_landing_airlocks(container: &Container, location_id: LocationId) -> Vec<LocationId> {
   container.locations.list_at(location_id)
       .flat_map(|id| container.rooms.get(id))
       .filter(|room| room.is_airlock)
       .map(|room| room.id)
       .collect()
}
