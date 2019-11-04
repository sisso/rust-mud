use super::comm;
use super::Outputs;
use super::domain::*;
use super::container::Container;
use super::mob::*;
use super::player::*;
use commons::PlayerId;

pub fn look(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) -> Result<(),()> {
    let player = container.players.get_player_by_id(player_id);

    outputs.private(
        player_id,
        comm::look_description(container, player.mob_id)?
    );

    Ok(())
}

pub fn say(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, msg: String) {
    let ctx = container.get_player_context(player_id);
    let player_msg = comm::say_you_say(&msg);
    let room_msg = comm::say_someone_said(&ctx.mob.label, &msg);

    outputs.private(player_id.clone(), player_msg);
    outputs.room(player_id.clone(), ctx.room.id, room_msg);
}

 pub fn mv(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, dir: Dir) -> Result<(),()> {
    let player = container.players.get_player_by_id(player_id);
    let location_id = container.locations.get(player.mob_id)?;
    let room = container.rooms.get(location_id)?;
    let exit_room_id = room.get_exit(&dir);

    match exit_room_id {
        Some(exit_room_id) => {
            let previous_room_id = location_id;
            // change mob place
            container.locations.set(player.mob_id,exit_room_id);

            let mob = container.mobs.get(player.mob_id)?;
            let mob_label = mob.label.as_str();

            let look = comm::look_description(&container, mob.id)?;
            let player_msg = format!("{}\n\n{}", comm::move_you_move(&dir), look);
            let enter_room_msg = comm::move_come(mob_label, &dir.inv());
            let exit_room_msg = comm::move_goes(mob_label, &dir);

            outputs.private(player_id, player_msg);
            outputs.room(player_id, previous_room_id, exit_room_msg);
            outputs.room(player_id, exit_room_id, enter_room_msg);
            Ok(())
        },
        None => {
            outputs.private(player_id, comm::move_not_possible(&dir));
            Err(())
        }
    }
}

pub fn attack(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId, target: MobId) -> Result<(),()> {
    let ctx = container.get_player_context(player_id);
    let target_mob = container.mobs.get(target)?;

    let player_msg = comm::attack_player_initiate(target_mob);
    let room_msg = comm::attack_mob_initiate_attack(&ctx.mob, &target_mob);

    let avatar_id = ctx.mob.id;
    let room_id = ctx.room.id;

    container.mobs.set_mob_attack_target(avatar_id, target);

    outputs.private(player_id, player_msg);
    outputs.room(player_id, room_id, room_msg);

    Ok(())
}

pub fn rest(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let ctx = container.get_player_context(player_id);
    let room_id = ctx.room.id;
    let mob_id = ctx.mob.id;

    if ctx.mob.is_combat() {
        outputs.private(player_id, comm::rest_fail_in_combat());
        return;
    }

    outputs.private(player_id, comm::rest_start());
    outputs.room(player_id, room_id,comm::rest_start_others(ctx.mob.label.as_str()));

    let mut mob = ctx.mob.clone();
    mob.set_action(MobAction::Resting, container.time.total);
    container.mobs.update(mob);
}

pub fn stand(container: &mut Container, outputs: &mut dyn Outputs, player_id: PlayerId) {
    let ctx = container.get_player_context(player_id);

    if ctx.mob.is_resting() {
        outputs.private(player_id, comm::stand_fail_not_resting());
        return;
    }

    outputs.private(player_id, comm::stand_up());
    outputs.room(player_id, ctx.room.id,comm::stand_up_others(ctx.mob.label.as_str()));

    let mut mob = ctx.mob.clone();
    mob.set_action(MobAction::Resting, container.time.total);
    container.mobs.update(mob);
}
