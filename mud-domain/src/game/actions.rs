use super::comm;
use super::container::Container;
use super::domain::*;
use super::mob::*;
use super::Outputs;
use commons::{AsResult, PlayerId, UResult, UERR, UOK, ObjId};
use crate::game::space_utils;
use std::process::id;
use logs::*;

pub fn look(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
) -> Result<(), ()> {
    let player = container.players.get(player_id);

    outputs.private(player_id, comm::look_description(container, player.mob_id)?);

    Ok(())
}

pub fn say(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: Option<PlayerId>,
    mob_id: MobId,
    msg: String,
) -> Result<(), ()> {
    let room_id = container.locations.get(mob_id).as_result()?;
    let mob_label = container.labels.get(mob_id).as_result()?;
    let player_msg = comm::say_you_say(&msg);
    let room_msg = comm::say_someone_said(mob_label.label.as_str(), &msg);

    outputs.private_opt(player_id, player_msg);
    outputs.room_opt(player_id, room_id, room_msg);

    Ok(())
}

// optional PlayerId
pub fn mv(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
    dir: Dir,
) -> UResult {
    let player = container.players.get(player_id);
    let mob_id = player.mob_id;
    let location_id = container.locations.get(mob_id).as_result()?;
    let room = container.rooms.get(location_id).as_result()?;
    let exit_room_id = room.get_exit(&dir);

    match exit_room_id {
        Some(exit_room_id) => {
            let previous_room_id = location_id;
            // change mob place
            container.locations.set(mob_id, exit_room_id);

            let label = container.labels.get(mob_id).unwrap();
            let mob_label = label.label.as_str();

            let look = comm::look_description(&container, mob_id).unwrap();
            let player_msg = format!("{}\n\n{}", comm::move_you_move(&dir), look);
            let enter_room_msg = comm::move_come(mob_label, &dir.inv());
            let exit_room_msg = comm::move_goes(mob_label, &dir);

            outputs.private(player_id, player_msg);
            outputs.room(player_id, previous_room_id, exit_room_msg);
            outputs.room(player_id, exit_room_id, enter_room_msg);
            Ok(())
        }
        None => {
            outputs.private(player_id, comm::move_not_possible(&dir));
            Err(())
        }
    }
}

// optional PlayerId
pub fn attack(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
    target_mob_id: MobId,
) -> Result<(), ()> {
    let player = container.players.get(player_id);
    let mob_id = player.mob_id;
    let room_id = container.locations.get(mob_id).as_result()?;

    let mob_label = container.labels.get_label(mob_id).unwrap();
    let target_label = container.labels.get_label(target_mob_id).unwrap();

    let player_msg = comm::attack_player_initiate(target_label);
    let room_msg = comm::attack_mob_initiate_attack(mob_label, target_label);

    outputs.private(player_id, player_msg);
    outputs.room(player_id, room_id, room_msg);

    container.mobs.set_mob_attack_target(mob_id, target_mob_id);

    Ok(())
}

// optional PlayerId
pub fn rest(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
) -> Result<(), ()> {
    let ctx = container.get_player_context(player_id);
    let room_id = ctx.room.id;
    let mob_id = ctx.mob.id;

    let total_time = container.time.total;

    if ctx.mob.is_combat() {
        outputs.private(player_id, comm::rest_fail_in_combat());
        return Err(());
    }

    let mob_label = container.labels.get_label(mob_id).unwrap();

    outputs.private(player_id, comm::rest_start());
    outputs.room(player_id, room_id, comm::rest_start_others(mob_label));
    container.mobs.update(mob_id, |mob| {
        mob.set_action(MobAction::Resting, total_time);
    });

    Ok(())
}

// optional PlayerId
pub fn stand(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: PlayerId,
) -> Result<(), ()> {
    let ctx = container.get_player_context(player_id);
    let mob_id = ctx.player.mob_id;
    let total_time = container.time.total;

    if ctx.mob.is_resting() {
        outputs.private(player_id, comm::stand_fail_not_resting());
        return Err(());
    }

    let mob_label = container.labels.get_label(mob_id).unwrap();

    outputs.private(player_id, comm::stand_up());
    outputs.room(player_id, ctx.room.id, comm::stand_up_others(mob_label));
    container.mobs.update(mob_id, |mob| {
        mob.set_action(MobAction::None, total_time);
    });

    Ok(())
}

pub fn enter(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: Option<PlayerId>,
    mob_id: MobId,
    arguments: &str
) -> UResult {
    let location_id = container.locations.get(mob_id).as_result()?;
    let candidates = space_utils::find_ships_at(container, location_id);
    let target =
        container.labels.search_codes(&candidates, arguments)
            .first().cloned();

    trace!("mob_id: {:?} at {:?}, candidates: {:?}, target: {:?}", mob_id, location_id, candidates, target);

    match target {
        Some(target) => {
            enter_do(container, outputs, player_id, mob_id, target)
        },

        None if arguments.is_empty() => {
            let codes = container.labels.resolve_codes(&candidates);
            outputs.private_opt(player_id, comm::enter_list(&codes));
            UERR
        }

        None => {
            let codes = container.labels.resolve_codes(&candidates);
            outputs.private_opt(player_id, comm::enter_invalid(arguments, &codes));
            UERR
        }
    }
}

pub fn enter_do(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: Option<PlayerId>,
    mob_id: MobId,
    target_id: ObjId,
) -> UResult {
    let current_location = container.locations
        .get(mob_id)
        .as_result()?;

    // find target room
    let candidate =
        space_utils::find_children_rooms_with_can_exit(container, target_id)
            .first()
            .cloned();

    match candidate {
        Some(location_id) => {
            let target_label = container.labels.get_label_f(target_id);
            let mob_label = container.labels.get_label_f(mob_id);

            // change mob place
            container.locations.set(mob_id, location_id);

            // emmit messages
            outputs.private_opt(player_id, comm::enter_player(target_label));
            outputs.private_opt(player_id, comm::look_description(&container, mob_id).unwrap());
            outputs.room_opt(player_id, current_location, comm::enter_others(mob_label, target_label));
            outputs.room_opt(player_id, location_id, comm::enter_others_other_side(mob_label));

            UOK
        }

        None => {
            outputs.private_opt(player_id, comm::enter_fail());
            UERR
        }
    }
}

pub fn out(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    player_id: Option<PlayerId>,
    mob_id: MobId,
) -> UResult {
    let location_id = container.locations.get(mob_id).as_result()?;

    let can_exit = container.rooms.get(location_id)
        .as_result()?
        .can_exit;

    if !can_exit {
        outputs.private_opt(player_id, comm::out_fail());
        return UERR;
    }

    let parents = container.locations.list_parents(location_id);
    let from_id = parents.get(0).cloned().as_result()?;
    let target_id = parents.iter()
        .filter(|&&id| container.rooms.exists(id))
        .next()
        .cloned();

    if let Some(target_id) = target_id {
        let from_label = container.labels.get_label_f(from_id);
        let mob_label = container.labels.get_label_f(mob_id);

        // change mob place
        container.locations.set(mob_id, target_id);

        // emmit messages
        outputs.private_opt(player_id, comm::out_player());
        outputs.private_opt(player_id, comm::look_description(&container, mob_id).unwrap());
        outputs.room_opt(player_id, location_id, comm::out_others(mob_label));
        outputs.room_opt(player_id, target_id, comm::out_others_other_side(mob_label, from_label));

        UOK
    } else {
        outputs.private_opt(player_id, comm::out_fail_bad_outside());
        UERR
    }
}
