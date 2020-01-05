use crate::game::system::SystemCtx;
use crate::game::item::ItemId;
use crate::game::comm;
use crate::errors::*;
use logs::*;

pub fn run(ctx: &mut SystemCtx) {
    ctx.container.items.list().into_iter().for_each(|id| {
        run_for(ctx, id)
            .as_failure()
            .err()
            .iter()
            .for_each(|error| {
                warn!("failure running system for {:?}: {:?}", id, error);
            })
    });
}

fn run_for(ctx: &mut SystemCtx, item_id: ItemId) -> Result<()> {
    let item = ctx.container.items.get(item_id).as_result()?;

    if let Some(decay) = item.decay {
        // TODO: Only decay items on ground?
        let location_id = ctx.container.locations.get(item.id).as_result()?;
        if ctx.container.rooms.exists(location_id) && decay.is_before(ctx.container.time.total) {
            info!("{:?} removed by decay", item.id);

            let label = ctx.container.labels.get_label_f(item.id);

            let msg = comm::item_body_disappears(label);
            ctx.outputs.broadcast(None, location_id, msg);
            ctx.container.remove(item_id);
            ctx.container.items.remove(item_id);
        }
    }

    Ok(())
}

