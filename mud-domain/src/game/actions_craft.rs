use crate::game::container::Container;
use crate::game::crafts::{CraftCommand, CraftId};
use crate::game::domain::Dir;
use crate::game::room::RoomId;
use crate::game::{comm, space_utils, Outputs};
use commons::{ObjId, PlayerId};
use logs::*;
use crate::game::mob::MobId;
use crate::errors::{Result, Error};

pub fn move_to(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    craft_id: CraftId,
    target_id: ObjId,
) -> Result<()> {
    container
        .crafts
        .set_command(craft_id, CraftCommand::MoveTo { target_id })
        .map(|ok| {
            debug!("move_to {:?} at {:?}", craft_id, target_id);
            outputs.private(mob_id, comm::space_move());
            ok
        })
        .map_err(|err| {
            outputs.private(mob_id, comm::space_move_invalid());
            err
        })
}

pub fn do_land_at(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    craft_id: CraftId,
    room_id: RoomId,
) -> Result<()> {
    debug!("landing {:?} at {:?}", craft_id, room_id);

    // find zone landing pad or airlock
    let landing_id = room_id;

    container.locations.set(craft_id, room_id);

    // collect labels
    let craft_label = container.labels.get_label(craft_id).unwrap();

    // emit events
    outputs.broadcast(None, craft_id, comm::space_land_complete());
    outputs.broadcast(None, landing_id, comm::space_land_complete_others(craft_label));

    Ok(())
}

pub fn do_launch(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    craft_id: CraftId,
) -> Result<()> {
    let parents = container.locations.list_parents(craft_id);
    let landing_pad_id = parents.get(0).cloned().unwrap();

    // search sector
    let sector_index = match parents.iter().position(|&id| container.sectors.exists(id)) {
        Some(index) => index,
        None => {
            outputs.private(mob_id, comm::space_launch_failed());
            return Err(Error::IllegalArgument);
        }
    };

    // collect launch position
    let sector_id = parents.get(sector_index).cloned().unwrap();
    let satellite_id = parents.get(sector_index - 1).cloned().unwrap();
    let pos = match container.pos.get_pos(satellite_id) {
        Some(pos) => pos,
        None => {
            outputs.private(mob_id, comm::space_launch_failed());
            return Err(Error::IllegalArgument);
        }
    };

    // put ship in position
    container.locations.set(craft_id, sector_id);
    container.pos.update(craft_id, pos);

    // collect labels
    let craft_label = container.labels.get_label(craft_id).unwrap();

    // emit events
    outputs.broadcast(None, craft_id, comm::space_launch_complete());
    outputs.broadcast(
        None,
        landing_pad_id,
        comm::space_launch_complete_others(craft_label),
    );

    Ok(())
}
