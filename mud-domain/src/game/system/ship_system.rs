use crate::game::astro_bodies::{AstroBody, AstroBodyKind};
use crate::game::container::Container;
use crate::game::ships::{LandState, LaunchState, MoveState, ShipCommand};
use crate::game::{astro_bodies, comm};
use crate::utils;
use crate::utils::geometry;
use commons::{DeltaTime, TotalTime};
use logs::*;

const DEFAULT_TIME: f32 = 1.0;
const TRANSFER_TIME: f32 = 2.0;
const SHIP_SPEED: f32 = 5.0;

pub fn tick(container: &mut Container) {
    let total_time = container.time.total;
    // let mut move_commands_complete = vec![];

    let ships = &mut container.ships;
    let locations = &mut container.locations;
    let astros = &mut container.astro_bodies;
    let labels = &mut container.labels;

    for ship in ships.list_all_mut() {
        let ship_id = ship.id;

        if ship.command.is_running(total_time) {
            // when command is running
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

                    astros
                        .update_orbit(ship_id, pos)
                        .expect("invalid state, can not move a ship that was not launch");
                }
                _ => {
                    // warn!("running an unexpected ship command {:?}", ship.command);
                }
            };
            continue;
        } else {
            // when command is not runnnig
            match &mut ship.command {
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
                    container.outputs.broadcast_all(None, ship_id, msg);
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
                    container.outputs.broadcast_all(None, ship_id, msg);
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
                    container.outputs.broadcast_all(None, ship_id, msg);
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
                    container.outputs.broadcast_all(None, ship_id, msg);
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
                    container.outputs.broadcast_all(None, ship_id, msg);
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
                    container.outputs.broadcast_all(None, ship_id, msg);
                }

                ShipCommand::Launch { target_id, state } => match state {
                    LaunchState::NotStarted => {
                        // update ship command
                        ship.command = ShipCommand::Launch {
                            target_id: *target_id,
                            state: LaunchState::Ignition {
                                complete_time: total_time + DeltaTime(DEFAULT_TIME),
                            },
                        };

                        // send messages
                        let msg = comm::space_launch_ignition();
                        container.outputs.broadcast_all(None, ship_id, msg);
                    }
                    LaunchState::Ignition { .. } => {
                        // set into parent orbit
                        let landing_pad_id = locations.get(ship.id).unwrap();
                        locations.set(ship.id, *target_id);

                        // TODO: create zero orbit and update until low orbit
                        // create astro body
                        let parent_body = astros.get(*target_id).unwrap();
                        let orbit_distance = parent_body.get_low_orbit();
                        let body = AstroBody::new(ship_id, orbit_distance, AstroBodyKind::Ship);

                        if let Some(old_value) = astros.upsert(body) {
                            warn!("{:?} obj with already existent astro_body when entering in orbit{:?}", ship_id, old_value);
                        };

                        // update command
                        ship.command = ShipCommand::Launch {
                            target_id: *target_id,
                            state: LaunchState::Ascending {
                                complete_time: total_time + DeltaTime(DEFAULT_TIME),
                            },
                        };

                        // emit events
                        let msg = comm::space_launch_ascending();
                        container.outputs.broadcast_all(None, ship_id, msg);

                        let craft_label = labels.get_label_f(ship_id);
                        container.outputs.broadcast_all(
                            None,
                            landing_pad_id,
                            comm::space_launch_complete_others(craft_label),
                        );
                    }
                    LaunchState::Ascending { .. } => {
                        // update ship command
                        ship.command = ShipCommand::Launch {
                            target_id: *target_id,
                            state: LaunchState::Circularization {
                                complete_time: total_time + DeltaTime(DEFAULT_TIME),
                            },
                        };

                        // send messages
                        let msg = comm::space_launch_burning_circularization();
                        container.outputs.broadcast_all(None, ship_id, msg);
                    }
                    LaunchState::Circularization { .. } => {
                        ship.command = ShipCommand::Idle;
                        let msg = comm::space_launch_complete();
                        container.outputs.broadcast_all(None, ship_id, msg);
                    }
                },

                ShipCommand::Land { target_id, state } => {
                    match state {
                        LandState::NotStarted => {
                            // update command
                            ship.command = ShipCommand::Land {
                                target_id: *target_id,
                                state: LandState::Running {
                                    stage: 0,
                                    complete_time: total_time + DeltaTime(DEFAULT_TIME),
                                },
                            };
                        }

                        // landing complete
                        LandState::Running { stage, .. } if *stage >= 4 => {
                            // update location
                            locations.set(ship_id, *target_id);

                            // remove astro
                            if astros.remove(ship_id).is_none() {
                                warn!("{:?} ship landed but was not in astro_bodies", ship_id);
                            }

                            // collect labels
                            let ship_label = labels.get_label(ship_id).unwrap();

                            // emit events
                            container.outputs.broadcast_all(
                                None,
                                ship_id,
                                comm::space_land_complete(),
                            );
                            container.outputs.broadcast(
                                None,
                                *target_id,
                                comm::space_land_complete_others(ship_label),
                            );

                            // set command to idle
                            ship.command = ShipCommand::Idle;
                        }

                        LandState::Running {
                            stage,
                            complete_time,
                        } => {
                            // send messages
                            let msg = match *stage {
                                0 => comm::space_land_retroburn(),
                                1 => comm::space_land_deorbit(),
                                2 => comm::space_land_aerobraking(),
                                3 => comm::space_land_approach(),
                                other => {
                                    warn!("{:?} unexpected land state stage {:?}", ship_id, other);
                                    continue;
                                }
                            };

                            // update command
                            *stage += 1;
                            *complete_time = total_time + DeltaTime(DEFAULT_TIME);

                            container.outputs.broadcast_all(None, ship_id, msg);
                        }
                    }
                }

                ShipCommand::Jump {
                    target_id: _target_id,
                    stage,
                    complete_time,
                } => {
                    let msg = match *stage {
                        0 => comm::space_jump_start(),
                        1 => comm::space_jump_recharging_capacitors(),
                        2 => comm::space_jump_do(),
                        3 => comm::space_jump_complete(),
                        other => {
                            warn!("{:?} unexpected jump stage {:?}", ship_id, other);
                            continue;
                        }
                    };

                    container.outputs.broadcast_all(None, ship_id, msg);

                    if *stage == 3 {
                        let location_id = locations.get(ship_id).unwrap();
                        let astro_location = astros.get(location_id).unwrap();
                        let target_jump_id = astro_location.jump_target_id.unwrap();

                        // get target jump point
                        locations.set(ship_id, target_jump_id);

                        // set command to idle
                        ship.command = ShipCommand::Idle;
                    } else {
                        *stage += 1;
                        *complete_time = total_time + DeltaTime(DEFAULT_TIME);
                    }
                }
            }
        }
    }
}
