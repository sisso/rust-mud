use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::ships::{MoveState, ShipCommand};
use crate::game::system::SystemCtx;
use crate::game::{astro_bodies, comm};
use crate::utils;
use crate::utils::geometry;
use commons::DeltaTime;
use logs::*;
use std::env::current_exe;

const TRANSFER_TIME: f32 = 2.0;
const SHIP_SPEED: f32 = 5.0;

pub fn tick(ctx: &mut SystemCtx) {
    let total_time = ctx.container.time.total;
    // let mut move_commands_complete = vec![];

    let ships = &mut ctx.container.ships;
    let locations = &mut ctx.container.locations;
    let astros = &mut ctx.container.astro_bodies;

    for ship in ships.list_all_mut() {
        let ship_id = ship.id;

        if ship.command.is_running(total_time) {
            match &ship.command {
                ShipCommand::MoveTo {
                    state:
                        MoveState::Drift {
                            from_distance,
                            to_distance,
                            start_time,
                            complete_time,
                        },
                    ..
                } => {
                    let pos = utils::lerp_2(
                        *from_distance,
                        *to_distance,
                        start_time.as_seconds_f64() as f32,
                        complete_time.as_seconds_f64() as f32,
                        total_time.as_seconds_f64() as f32,
                    );

                    astros.update_orbit(ship_id, pos).unwrap();
                }

                _ => {}
            };
            continue;
        } else {
            match &ship.command {
                ShipCommand::Idle => {}

                ShipCommand::MoveTo {
                    target_id,
                    state: MoveState::NotStarted,
                } => {
                    ship.command = ShipCommand::MoveTo {
                        target_id: *target_id,
                        state: MoveState::Alignment {
                            complete_time: total_time.add(DeltaTime(TRANSFER_TIME)),
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
                            complete_time: total_time.add(DeltaTime(TRANSFER_TIME)),
                        },
                    };

                    let msg = comm::space_fly_alignment_complete_start_ejection_burn();
                    ctx.outputs.broadcast_all(None, ship_id, msg);
                }

                ShipCommand::MoveTo {
                    target_id,
                    state: MoveState::EjectionBurn { .. },
                } => {
                    let travel =
                        match astro_bodies::travel_plan(locations, astros, ship_id, *target_id) {
                            Ok(travel_plan) => travel_plan,

                            Err(err) => {
                                warn!(
                                    "{:?} can not find travel plan to {:?}: {:?}",
                                    ship_id, target_id, err
                                );
                                continue;
                            }
                        };

                    let ship_speed = SHIP_SPEED;
                    let travel_time = DeltaTime(travel.total_distance / ship_speed);

                    locations.set(ship_id, travel.root_body_id);

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
                            complete_time: total_time.add(DeltaTime(TRANSFER_TIME)),
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
                            complete_time: total_time.add(DeltaTime(TRANSFER_TIME)),
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
                    // TODO: find right orbital distance from body
                    astros.update_orbit(ship_id, 0.1).unwrap();

                    let msg = comm::space_fly_complete();
                    ctx.outputs.broadcast_all(None, ship_id, msg);
                }
            }
        }
    }
}
