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
                let travel = match astro_bodies::travel_plan(locations, astros, ship_id, *target_id)
                {
                    Ok(travel_plan) => travel_plan,

                    Err(err) => {
                        warn!(
                            "{:?} can not find travel plan to {:?}: {:?}",
                            ship_id, target_id, err
                        );
                        continue;
                    }
                };

                let ship_speed = 10.0;
                let travel_time = DeltaTime(travel.total_distance / ship_speed);

                locations.set(ship_id, travel.reference_body_id);

                ship.command = ShipCommand::MoveTo {
                    target_id: *target_id,
                    state: MoveState::Drift {
                        from_distance: travel.from_distance,
                        to_distance: travel.to_distance,
                        start_time: total_time,
                        complete_time: total_time.add(travel_time),
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
