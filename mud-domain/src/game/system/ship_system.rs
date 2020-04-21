use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::ships::{MoveState, ShipCommand};
use crate::game::system::SystemCtx;
use crate::game::{astro_bodies, comm};
use crate::utils::geometry;
use commons::DeltaTime;
use logs::*;

pub fn tick(ctx: &mut SystemCtx) {
    let total_time = ctx.container.time.total;
    // let mut move_commands_complete = vec![];

    let ships = &mut ctx.container.ships;
    let locations = &mut ctx.container.locations;
    let astros = &mut ctx.container.astro_bodies;

    for ship in ships.list_all_mut() {
        if ship.command.is_running(total_time) {
            continue;
        }

        let ship_id = ship.id;

        match &ship.command {
            ShipCommand::Idle => {}

            ShipCommand::MoveTo {
                target_id,
                state: MoveState::NotStarted,
            } => {
                ship.command = ShipCommand::MoveTo {
                    target_id: *target_id,
                    state: MoveState::Alignment {
                        complete_time: total_time.add(DeltaTime(1.0)),
                    },
                };

                let msg = comm::space_fly_start();
                ctx.outputs.broadcast_all(None, ship_id, msg);
            }

            ShipCommand::MoveTo {
                target_id,
                state: MoveState::Alignment { .. },
            } => {
                ship.command = ShipCommand::MoveTo {
                    target_id: *target_id,
                    state: MoveState::EjectionBurn {
                        complete_time: total_time.add(DeltaTime(1.0)),
                    },
                };

                let msg = comm::space_fly_alignment_complete_start_ejection_burn();
                ctx.outputs.broadcast_all(None, ship_id, msg);
            }

            ShipCommand::MoveTo {
                target_id,
                state: MoveState::EjectionBurn { .. },
            } => {
                let star_id = match astro_bodies::find_start_of(locations, astros, ship_id) {
                    Some(id) => id,
                    None => {
                        warn!("{:?} can not find star in parent locations", ship_id);
                        continue;
                    }
                };
                locations.set(ship_id, star_id);

                ship.command = ShipCommand::MoveTo {
                    target_id: *target_id,
                    state: MoveState::Drift {
                        from_distance: 0.0,
                        to_distance: 0.0,
                        start_time: total_time,
                        complete_time: total_time.add(DeltaTime(1.0)),
                    },
                };

                let msg = comm::space_fly_ejection_burn_complete();
                ctx.outputs.broadcast_all(None, ship_id, msg);
            }

            ShipCommand::MoveTo {
                target_id,
                state: MoveState::Drift { .. },
            } => {
                ship.command = ShipCommand::MoveTo {
                    target_id: *target_id,
                    state: MoveState::RetroBurn {
                        complete_time: total_time.add(DeltaTime(1.0)),
                    },
                };

                let msg = comm::space_fly_drift_complete();
                ctx.outputs.broadcast_all(None, ship_id, msg);
            }

            ShipCommand::MoveTo {
                target_id,
                state: MoveState::RetroBurn { .. },
            } => {
                ship.command = ShipCommand::MoveTo {
                    target_id: *target_id,
                    state: MoveState::OrbitSync {
                        complete_time: total_time.add(DeltaTime(1.0)),
                    },
                };

                let msg = comm::space_fly_retroburn_complete_start_orbital_sync();
                ctx.outputs.broadcast_all(None, ship_id, msg);
            }

            ShipCommand::MoveTo {
                target_id,
                state: MoveState::OrbitSync { .. },
            } => {
                locations.set(ship_id, *target_id);

                ship.command = ShipCommand::Idle;

                let msg = comm::space_fly_complete();
                ctx.outputs.broadcast_all(None, ship_id, msg);
            }
        }
    }
}
