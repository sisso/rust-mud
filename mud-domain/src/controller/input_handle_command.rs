use crate::controller::ViewHandleCtx;
use crate::errors::*;
use crate::game::{actions_command, comm};
use crate::utils::strinput::StrInput;
use commons::ObjId;
use logs::*;

pub const COMMAND_SEP: char = ':';

pub fn list_commands(ctx: &mut ViewHandleCtx) -> Result<()> {
    let commandables = actions_command::list_commandable(ctx.container, ctx.mob_id)?;
    let labels = ctx
        .container
        .labels
        .resolve(&commandables.iter().map(|i| *i).collect());
    ctx.container
        .outputs
        .private(ctx.mob_id, comm::list_commandables(&labels));
    Ok(())
}

pub fn list_commands_for(_ctx: &mut ViewHandleCtx, target: &str) -> Result<()> {
    // let target_id = match find_command_target(ctx, target) {
    //     Some(target_id) => target_id,
    //     None => return Ok(()),
    // };
    //
    // let label = ctx.container.labels.get_label_f(target_id);
    //
    // let commands = actions_command::list_commands_for(ctx.container, target_id)?;
    // ctx.container
    //     .outputs
    //     .private(ctx.mob_id, comm::list_commands(label, &commands));
    // Ok(())
    Err(Error::NotImplementedException)
}

pub fn set_command(ctx: &mut ViewHandleCtx, target: &str, command: StrInput) -> Result<()> {
    // let target_id = match find_command_target(ctx, target) {
    //     Some(target_id) => target_id,
    //     None => return Ok(()),
    // };
    //
    // if command.plain_arguments().has_command("follow me") {
    //     set_command_follow_me(ctx, target_id)
    // } else if command.has_command("extract") {
    //     set_command_extract(ctx, target_id)
    // } else if command.has_command("haul") {
    //     set_command_haul(ctx, target_id, command)
    // } else {
    //     ctx.container.outputs.private(
    //         ctx.mob_id,
    //         comm::command_invalid_for_target(target, command.plain_arguments()),
    //     );
    //     Ok(())
    // }
    Err(Error::NotImplementedException)
}

fn set_command_haul(ctx: &mut ViewHandleCtx, target_id: ObjId, command: StrInput) -> Result<()> {
    // let args = command.parse_arguments();
    // if args.len() != 2 {
    //     ctx.container
    //         .outputs
    //         .private(ctx.mob_id, comm::command_move_invalid_argument());
    //     return Ok(());
    // }
    //
    // let (from_id, to_id) = match (args[0].parse::<u32>(), args[1].parse::<u32>()) {
    //     (Ok(id_from), Ok(id_to)) => {
    //         match (
    //             ctx.container.rooms.exists(id_from.into()),
    //             ctx.container.rooms.exists(id_to.into()),
    //         ) {
    //             (true, true) => (id_from, id_to),
    //             (true, false) => {
    //                 ctx.container
    //                     .outputs
    //                     .private(ctx.mob_id, comm::command_haul_from_not_found(args[1]));
    //                 return OK(());
    //             }
    //             (false, _) => {
    //                 ctx.container
    //                     .outputs
    //                     .private(ctx.mob_id, comm::command_haul_from_not_found(args[0]));
    //                 return OK(());
    //             }
    //         }
    //     }
    //     _ => {
    //         ctx.container
    //             .outputs
    //             .private(ctx.mob_id, comm::command_haul_invalid_argument());
    //         return OK(());
    //     }
    // };
    //
    // actions_command::set_command_haul(ctx.container, target_id, from_id, to_id)?;
    //
    // Ok(())
    Err(Error::NotImplementedException)
}

fn set_command_follow_me(ctx: &mut ViewHandleCtx, target_id: ObjId) -> Result<()> {
    actions_command::set_command_follow(ctx.container, target_id, ctx.mob_id)?;

    let label = ctx.container.labels.get_label_f(target_id);
    ctx.container
        .outputs
        .private(ctx.mob_id, comm::command_follow_me_ack(label));
    Ok(())
}

fn set_command_extract(ctx: &mut ViewHandleCtx, target_id: ObjId) -> Result<()> {
    let location_id = ctx
        .container
        .locations
        .get(ctx.mob_id)
        .as_result_str("mob has no location")?;

    let extractable: Vec<ObjId> = ctx
        .container
        .locations
        .list_at(location_id)
        .filter(|id| ctx.container.extractables.exist(*id))
        .collect();

    if extractable.len() > 1 {
        warn!("extract command selection for multiple targets not implemented");
    }

    match extractable.into_iter().next() {
        Some(extractable_id) => {
            actions_command::set_command_extract(
                ctx.container,
                target_id,
                location_id,
                extractable_id,
            )?;

            let label = ctx.container.labels.get_label_f(target_id);
            ctx.container
                .outputs
                .private(ctx.mob_id, comm::command_extract_ack(label));
        }

        None => {
            ctx.container
                .outputs
                .private(ctx.mob_id, comm::command_extract_fail_no_extractable());
        }
    }

    Ok(())
}

pub fn command(ctx: &mut ViewHandleCtx, input: &StrInput) -> Result<()> {
    let plain_args = input.plain_arguments();
    if plain_args.is_empty() {
        list_commands(ctx)
    } else {
        let parts: Vec<_> = plain_args
            .split(COMMAND_SEP)
            .into_iter()
            .map(|i| i.trim())
            .collect();

        if parts.len() == 1 {
            list_commands_for(ctx, parts[0])
        } else if parts.len() == 2 {
            set_command(ctx, parts[0], StrInput(parts[1]))
        } else {
            ctx.container
                .outputs
                .private(ctx.mob_id, comm::command_invalid());
            Ok(())
        }
    }
}
