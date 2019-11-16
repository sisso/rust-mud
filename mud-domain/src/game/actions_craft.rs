use crate::game::crafts::{CraftId, CraftCommand};
use commons::{ObjId, PlayerId, UErr};
use crate::game::{Outputs, comm, space_utils, builder};
use crate::game::container::Container;
use crate::game::room::RoomId;
use crate::game::domain::Dir;
use logs::*;

pub fn move_to(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, craft_id: CraftId, target_id: ObjId) -> Result<(), ()> {
    container.crafts.set_command(craft_id, CraftCommand::MoveTo { target_id })
        .map(|ok| {
            outputs.private(player_id, comm::space_move());
            ok
        })
        .map_err(|err| {
            outputs.private(player_id, comm::space_move_invalid());
            err
        })
}

pub fn do_land_at(container: &mut Container, outputs: &mut dyn Outputs, craft_id: CraftId, room_id: RoomId) -> Result<(), ()> {
    debug!("landing {:?} at {:?}", craft_id, room_id);

    // find zone landing pad or airlock
    let landing_id = room_id;

    // find craft airlock
    let craft_airlock_candidates = space_utils::get_landing_airlocks(container, craft_id);
    let craft_airlock_id = craft_airlock_candidates.first().cloned().ok_or(())?;

    trace!("landing {:?} at {:?}, landing pad: {:?}, craft airlock: {:?}", craft_id, room_id, landing_id, craft_airlock_id);

    let location_id = container.locations.get(room_id)?;
    container.locations.set(craft_id, location_id);

    // connect the craft with room
    container.rooms.add_portal(landing_id, craft_airlock_id, Dir::Enter);

    outputs.zone_all(craft_id, comm::space_land_complete());

    Ok(())
}
