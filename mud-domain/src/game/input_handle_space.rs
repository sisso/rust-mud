use commons::{PlayerId};
use crate::game::container::Container;
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::game::crafts::CraftId;
use crate::game::comm::{ShowStarmapDesc, ShowStarmapDescKind};
use logs::*;

pub fn show_starmap(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, mob_id: MobId) -> Result<(),()> {
    let craft_id = match get_craft(container, mob_id) {
        Ok(craft_id) => craft_id,
        Err(()) => {
            outputs.private(player_id, comm::space_not_in_craft());
            return Err(())
        }
    };

    let craft_location = container.locations.get(craft_id)?;
    if !container.sectors.exists(craft_location) {
        outputs.private(player_id, comm::space_not_in_craft());
        return Err(())
    }

    // get type and pos and what we are
    let objects = container.locations.list_at(craft_location).flat_map(|id| {
        let pos = container.pos.get(id).ok();
        let is_craft= container.crafts.exists(id);
        let is_planet = container.planets.exists(id);

        match pos {
            Some(pos) if is_craft => Some(ShowStarmapDesc {
                kind: ShowStarmapDescKind::Craft,
                pos: pos.pos,
                me: id == craft_id
            }),
            Some(pos) if is_planet => Some(ShowStarmapDesc {
                kind: ShowStarmapDescKind::Planet,
                pos: pos.pos,
                me: false
            }),
            _ => None
        }
    }).collect();

    outputs.private(player_id, comm::space_show_starmap(&objects));

    Ok(())
}

fn get_craft(container: &mut Container, mob_id: MobId) -> Result<CraftId, ()> {
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


