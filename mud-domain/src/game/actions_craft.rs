use crate::game::container::Container;
use crate::game::crafts::{CraftCommand, CraftId};
use crate::game::domain::Dir;
use crate::game::room::RoomId;
use crate::game::{comm, space_utils, Outputs};
use commons::{ObjId, PlayerId, UResult, UERR, UOK};
use logs::*;

pub fn move_to(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
    craft_id: CraftId,
    target_id: ObjId,
) -> UResult {
    container
        .crafts
        .set_command(craft_id, CraftCommand::MoveTo { target_id })
        .map(|ok| {
            outputs.private(player_id, comm::space_move());
            ok
        })
        .map_err(|err| {
            outputs.private(player_id, comm::space_move_invalid());
            err
        })
}

pub fn do_land_at(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    craft_id: CraftId,
    room_id: RoomId,
) -> UResult {
    debug!("landing {:?} at {:?}", craft_id, room_id);

    // find zone landing pad or airlock
    let landing_id = room_id;

    // find craft airlock
    let craft_airlock_candidates = space_utils::get_landing_airlocks(container, craft_id);
    let craft_airlock_id = craft_airlock_candidates.first().cloned().ok_or(())?;

    trace!(
        "landing {:?} at {:?}, landing pad: {:?}, craft airlock: {:?}",
        craft_id,
        room_id,
        landing_id,
        craft_airlock_id
    );

    container.locations.set(craft_id, room_id);

    // connect the craft with room
    container
        .rooms
        .add_portal(landing_id, craft_airlock_id, Dir::Enter);

    // collect labels
    let craft_label = container.labels.get_label(craft_id).unwrap();

    // emit events
    outputs.zone_all(craft_id, comm::space_land_complete());
    outputs.room_all(landing_id, comm::space_land_complete_others(craft_label));

    Ok(())
}

pub fn do_launch(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
    craft_id: CraftId,
) -> UResult {
    let parents = container.locations.list_parents(craft_id);

    // search sector
    let sector_index = match parents.iter().position(|&id| container.sectors.exists(id)) {
        Some(index) => index,
        None => {
            outputs.private(player_id, comm::space_launch_failed());
            return UERR;
        }
    };

    // collect launch position
    let sector_id = parents.get(sector_index).cloned().unwrap();
    let satellite_id = parents.get(sector_index - 1).cloned().unwrap();
    let pos = match container.pos.get_pos(satellite_id) {
        Some(pos) => pos,
        None => {
            outputs.private(player_id, comm::space_launch_failed());
            return UERR;
        }
    };

    // search for landing pad and airlock connection
    let landing_pad_id = parents.get(0).cloned().unwrap();
    let craft_airlock_candidates = space_utils::get_landing_airlocks(container, craft_id);
    let (airlock_id, exit_dir) = craft_airlock_candidates
        .iter()
        .find_map(|&room_id| {
            container
                .rooms
                .exists_exits(room_id, landing_pad_id)
                .map(|exit_dir| (room_id, exit_dir))
        })
        .ok_or(())?;

    // remove portal
    container
        .rooms
        .remove_portal(airlock_id, landing_pad_id, exit_dir);

    // put ship in position
    container.locations.set(craft_id, sector_id);
    container.pos.update(craft_id, pos);

    // collect labels
    let craft_label = container.labels.get_label(craft_id).unwrap();

    // emit events
    outputs.zone_all(craft_id, comm::space_launch_complete());
    outputs.room_all(
        landing_pad_id,
        comm::space_launch_complete_others(craft_label),
    );

    UOK
}
