use crate::game::comm;
use crate::game::ships::ShipCommand;
use crate::utils::geometry;
use crate::game::system::SystemCtx;
use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use logs::*;

pub fn tick(ctx: &mut SystemCtx) {
    let total_time = ctx.container.time.total;
    let mut move_commands_complete = vec![];

    for ship in ctx.container.ships.list_all() {
        match ship.command {
            ShipCommand::Idle => {},
            ShipCommand::MoveTo { target_id, arrival_time } => {
                if arrival_time.is_after(total_time) {
                    // not yea
                } else {
                    move_commands_complete.push((ship.id, true, target_id));
                }
            }
        }
    }

    for (ship_id, success, target_id) in move_commands_complete {
        ctx.container.ships.set_command(ship_id, ShipCommand::Idle).unwrap();

        let msg = if success {
            ctx.container.locations.set(ship_id, target_id);

            let low_orbit = ctx.container.astro_bodies.get(target_id).unwrap().get_low_orbit();

            ctx.container.astro_bodies.update(AstroBody {
                id: ship_id,
                orbit_distance: low_orbit,
                kind: AstroBodyKind::Ship
            }).unwrap();

            comm::space_command_complete()
        } else {
            comm::space_command_failed()
        };

        ctx.outputs.broadcast_all(None, ship_id, msg);
    }
}
