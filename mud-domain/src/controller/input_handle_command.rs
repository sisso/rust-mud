use crate::controller::ViewHandleCtx;
use crate::errors::Result;
use crate::game::{actions_command, comm};
use crate::utils::strinput::StrInput;
use logs::*;

pub fn list_commands(ctx: &mut ViewHandleCtx) -> Result<()> {
    let commandables = actions_command::list_commandable(ctx.container, ctx.mob_id)?;
    let labels = ctx
        .container
        .labels
        .resolve(&commandables.iter().map(|i| i.id).collect());
    ctx.container
        .outputs
        .private(ctx.mob_id, comm::list_commandables(&labels));
    Ok(())
}

pub fn command(ctx: &mut ViewHandleCtx, input: &StrInput) -> Result<()> {
    let plain_args = input.plain_arguments();
    if plain_args.is_empty() {
        return list_commands(ctx);
    }
    todo!()
}
