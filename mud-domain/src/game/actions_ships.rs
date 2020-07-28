use crate::errors::{AsResult, Error, Result};
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::container::Container;
use crate::game::domain::Dir;
use crate::game::mob::MobId;
use crate::game::outputs::Outputs;
use crate::game::room::RoomId;
use crate::game::ships::{ShipCommand, ShipId};
use crate::game::{astro_bodies, comm, space_utils};
use commons::{DeltaTime, ObjId, PlayerId};
use logs::*;

/// Assume that all arguments are correct
pub fn move_to(
    container: &mut Container,
    mob_id: MobId,
    ship_id: ShipId,
    target_id: ObjId,
) -> Result<()> {
    container
        .ships
        .set_command(ship_id, ShipCommand::move_to(target_id))
        .map(|ok| {
            debug!("move_to {:?} at {:?}", ship_id, target_id);
            container.outputs.private(mob_id, comm::space_move());
            ok
        })
        .map_err(|err| {
            container
                .outputs
                .private(mob_id, comm::space_move_invalid());
            err
        })
}

pub fn do_land_at(
    container: &mut Container,
    ship_id: ShipId,
    landing_room_id: RoomId,
) -> Result<()> {
    match container
        .ships
        .set_command(ship_id, ShipCommand::land(landing_room_id))
    {
        Ok(()) => {
            let msg = comm::space_land_started();
            container.outputs.broadcast_all(None, ship_id, msg);
            Ok(())
        }

        Err(e) => {
            let msg = comm::space_land_invalid();
            container.outputs.broadcast_all(None, ship_id, msg);
            Err(e)
        }
    }
}

pub fn do_launch(container: &mut Container, mob_id: MobId, ship_id: ShipId) -> Result<()> {
    let can_launch = space_utils::can_ship_launch(container, ship_id);
    let target_orbit_id = space_utils::get_ship_body_on_lunch(container, ship_id);

    match (can_launch, target_orbit_id) {
        (true, Some(target_id)) => {
            if container
                .ships
                .set_command(ship_id, ShipCommand::launch(target_id))
                .is_ok()
            {
                container
                    .outputs
                    .private(mob_id, comm::space_launch_prepare());
                Ok(())
            } else {
                container
                    .outputs
                    .private(mob_id, comm::space_launch_failed());
                Err(Error::InvalidArgumentFailure)
            }
        }
        _ => {
            container
                .outputs
                .private(mob_id, comm::space_launch_failed());
            Err(Error::InvalidArgumentFailure)
        }
    }
}

pub fn do_jump(container: &mut Container, mob_id: MobId, ship_id: ShipId) -> Result<()> {
    let location_id = container.locations.get(ship_id).as_result()?;
    let astro_location = container.astro_bodies.get(location_id).as_result()?;

    if astro_location.kind != AstroBodyKind::JumpGate {
        warn!("{:?} jump {:?} fail, invalid paretn", mob_id, ship_id);
        container.outputs.private(mob_id, comm::space_jump_failed());
        return Err(Error::InvalidArgumentFailure);
    }

    let target_jump_id = match astro_location.jump_target_id {
        Some(id) => id,
        None => {
            warn!(
                "{:?} can not jump, jump {:?} has no target",
                mob_id, astro_location
            );
            container.outputs.private(mob_id, comm::space_jump_failed());
            return Err(Error::InvalidArgumentFailure);
        }
    };

    // get target jump point
    container.locations.set(ship_id, target_jump_id);

    // emit events
    container
        .outputs
        .broadcast_all(None, ship_id, comm::space_jump_complete());

    Ok(())
}
