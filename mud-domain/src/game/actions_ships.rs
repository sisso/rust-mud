use crate::errors::{AsResult, Error, Result};
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::container::Container;
use crate::game::domain::Dir;
use crate::game::mob::MobId;
use crate::game::room::RoomId;
use crate::game::ships::{ShipCommand, ShipId};
use crate::game::{comm, space_utils, Outputs};
use commons::{DeltaTime, ObjId, PlayerId};
use logs::*;

/// Assume that all arguments are correct
pub fn move_to(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    craft_id: ShipId,
    target_id: ObjId,
) -> Result<()> {
    // TODO compute a proper distance
    let travel_time = DeltaTime(5.0);
    let arrival_time = container.time.total + travel_time;

    container
        .ships
        .set_command(
            craft_id,
            ShipCommand::MoveTo {
                target_id,
                arrival_time,
            },
        )
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
    craft_id: ShipId,
    room_id: RoomId,
) -> Result<()> {
    debug!("landing {:?} at {:?}", craft_id, room_id);

    // find zone landing pad or airlock
    let landing_id = room_id;

    container.locations.set(craft_id, room_id);

    // collect labels
    let craft_label = container.labels.get_label(craft_id).unwrap();

    // emit events
    outputs.broadcast_all(None, craft_id, comm::space_land_complete());
    outputs.broadcast(
        None,
        landing_id,
        comm::space_land_complete_others(craft_label),
    );

    Ok(())
}

pub fn do_launch(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    ship_id: ShipId,
) -> Result<()> {
    let parents = container.locations.list_parents(ship_id);
    let landing_pad_id = parents.get(0).cloned().unwrap();

    // search current body
    let parent_body = parents
        .iter()
        .flat_map(|&id| container.space_body.get(id))
        .next();

    let parent_body = match parent_body {
        Some(body) => body,
        None => {
            warn!(
                "{:?} launch {:?} fail, ship is not in astrobody",
                mob_id, ship_id
            );
            outputs.private(mob_id, comm::space_launch_failed());
            return Err(Error::InvalidArgumentFailure);
        }
    };

    let parent_body_id = parent_body.id;
    let orbit_distance = parent_body.get_low_orbit();

    let body = AstroBody::new(ship_id, orbit_distance, AstroBodyKind::Ship);

    // put ship in low orbit
    if let Err(error) = container.space_body.insert(body) {
        warn!(
            "{:?} launch {:?} fail to set ship orbit: {:?}",
            mob_id, ship_id, error
        );
        outputs.private(mob_id, comm::space_launch_failed());
        return Err(Error::InvalidArgumentFailure);
    };

    container.locations.set(ship_id, parent_body_id);

    // collect labels
    let craft_label = container.labels.get_label(ship_id).unwrap();

    // emit events
    outputs.broadcast_all(None, ship_id, comm::space_launch_complete());
    outputs.broadcast_all(
        None,
        landing_pad_id,
        comm::space_launch_complete_others(craft_label),
    );

    Ok(())
}

pub fn do_jump(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    ship_id: ShipId,
) -> Result<()> {
    let location_id = container.locations.get(ship_id).as_result()?;
    let astro_location = container.space_body.get(location_id).as_result()?;

    if astro_location.kind != AstroBodyKind::JumpGate {
        warn!("{:?} jump {:?} fail, invalid paretn", mob_id, ship_id);
        outputs.private(mob_id, comm::space_jump_failed());
        return Err(Error::InvalidArgumentFailure);
    }

    let target_jump_id = match astro_location.jump_target_id {
        Some(id) => id,
        None => {
            warn!(
                "{:?} can not jump, jump {:?} has no target",
                mob_id, astro_location
            );
            outputs.private(mob_id, comm::space_jump_failed());
            return Err(Error::InvalidArgumentFailure);
        }
    };

    // get target jump point
    container.locations.set(ship_id, target_jump_id);

    // emit events
    outputs.broadcast_all(None, ship_id, comm::space_jump_complete());

    Ok(())
}
