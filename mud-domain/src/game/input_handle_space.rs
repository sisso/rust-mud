use commons::{PlayerId, ObjId};
use crate::game::container::Container;
use crate::game::{actions_craft};
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::game::crafts::CraftId;
use crate::game::comm::{SurfaceDesc, ShowStarmapDescKind};
use logs::*;
use crate::utils::text::search_label;

pub fn show_starmap(container: &Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    let (craft_id, craft_location) = get_craft_and_location(container, outputs, player_id, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, craft_location);
    outputs.private(player_id, comm::space_show_sectormap(&objects));
    Ok(())
}

pub fn move_list_targets(container: & Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    let (craft_id, craft_location) = get_craft_and_location(container, outputs, player_id, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, craft_location);
    outputs.private(player_id, comm::space_show_move_targets(&objects));
    Ok(())
}

pub fn move_to(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId, input: Vec<&str>) -> Result<(),()> {
    let (craft_id, craft_location) = get_craft_and_location(container, outputs, player_id, mob_id)?;
    input.get(1).ok_or(()).and_then(|label| {
        find_surface_target(container, craft_location, label)
    }).and_then(|target_id| {
        actions_craft::move_to(container, outputs, player_id, craft_id, target_id)
    }).map_err(|_| {
        outputs.private(player_id, comm::space_move_invalid());
    })
}

fn find_surface_target(container: &mut Container, craft_location: ObjId, label: &str) -> Result<ObjId,()> {
    let candidates = container.locations.list_at(craft_location).collect::<Vec<_>>();
    let founds = container.labels.search_codes(&candidates, label);
    founds.first().cloned().ok_or(())
}

fn get_objects_in_surface(container: &Container, craft_id: ObjId, craft_location: ObjId) -> Vec<SurfaceDesc> {
    let objects = container.locations.list_at(craft_location).flat_map(|id| {
        let label = container.labels.get_label_f(id);
        let pos = container.pos.get(id).ok();
        let is_craft = container.crafts.exists(id);
        let is_planet = container.planets.exists(id);

        match pos {
            Some(pos) if is_craft => Some(SurfaceDesc {
                kind: ShowStarmapDescKind::Craft,
                pos: pos.pos,
                me: id == craft_id,
                label: label.to_string(),
            }),
            Some(pos) if is_planet => Some(SurfaceDesc {
                kind: ShowStarmapDescKind::Planet,
                pos: pos.pos,
                me: false,
                label: label.to_string(),
            }),
            _ => None
        }
    }).collect();
    objects
}

fn get_craft_and_location(container: &Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(CraftId, ObjId),()> {
    let craft_id = match get_craft(container, mob_id) {
        Ok(craft_id) => craft_id,
        Err(()) => {
            outputs.private(player_id, comm::space_not_in_craft());
            return Err(());
        }
    };

    let craft_location = container.locations.get(craft_id)?;
    if !container.sectors.exists(craft_location) {
        outputs.private(player_id, comm::space_not_in_craft());
        return Err(());
    }

    return Ok((craft_id, craft_location));
}

fn get_craft(container: &Container, mob_id: MobId) -> Result<CraftId, ()> {
    let room_id = container.locations.get(mob_id)?;
    if !container.rooms.exists(room_id) {
        return Err(());
    }

    let craft_id = container.locations.get(room_id)?;
    if !container.crafts.exists(craft_id) {
        return Err(());
    }

    Ok(craft_id)
}


