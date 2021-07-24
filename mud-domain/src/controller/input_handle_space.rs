use crate::controller::ViewHandleCtx;
use crate::errors::{AsResult, Error, Result};
use crate::game::actions_ships;
use crate::game::actions_ships::{do_jump, do_land_at, do_launch};
use crate::game::astro_bodies::AstroBody;
use crate::game::comm;
use crate::game::comm::ShowSectorTreeBody;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::game::obj::Obj;
use crate::game::space_utils::*;
use crate::utils::strinput::StrInput;
use crate::utils::text;
use commons::{ObjId, PlayerId};
use logs::*;

pub fn show_startree(container: &mut Container, mob_id: MobId) -> Result<()> {
    let (ship_id, sector_id) = get_ship_and_sector(container, mob_id)?;
    let bodies = find_showsector_bodies(container, sector_id, Some(ship_id));
    let sector_label = container.labels.get_label_f(sector_id);
    // trace!("{:?} at {:?} on sector {:?} can view {:?}", mob_id, ship_id, sector_id, bodies);
    let msg = comm::show_sectortree(sector_id, sector_label, &bodies);
    container.outputs.private(mob_id, msg);
    Ok(())
}

/// deprecated
pub fn show_surface_map(container: &mut Container, mob_id: MobId) -> Result<()> {
    let (craft_id, sector_id) = get_ship_and_sector(container, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, sector_id);
    let msg = comm::show_surface_map(&objects);
    container.outputs.private(mob_id, msg);
    Ok(())
}

pub fn move_list_targets(container: &mut Container, mob_id: MobId) -> Result<()> {
    let (ship_id, sector_id) = get_ship_and_sector(container, mob_id)?;
    let targets = find_move_targets(container, sector_id, ship_id)
        .into_iter()
        .map(|body| to_showsectortreebody(container, Some(ship_id), body))
        .collect();

    let msg = comm::space_show_move_targets(&targets);
    container.outputs.private(mob_id, msg);

    Ok(())
}

/// Search for space bodies that can be used to "move"
fn find_move_targets(container: &Container, sector_id: ObjId, ship_id: ObjId) -> Vec<&AstroBody> {
    find_space_bodies(container, sector_id)
        .into_iter()
        .filter(|i| i.id != ship_id)
        .collect()
}

pub fn move_to(container: &mut Container, mob_id: MobId, input: &StrInput) -> Result<()> {
    let (ship_id, sector_id) = get_ship_and_sector(container, mob_id)?;
    let targets = find_move_targets(container, sector_id, ship_id)
        .into_iter()
        .map(|i| i.id)
        .collect();

    let founds = container.labels.search(&targets, input.plain_arguments());

    match founds.get(0) {
        Some(&target_id) => actions_ships::move_to(container, mob_id, ship_id, target_id),

        None => {
            container
                .outputs
                .private(mob_id, comm::space_move_invalid());
            Err(Error::InvalidArgumentFailure)
        }
    }
}

pub fn land_list(container: &mut Container, mob_id: MobId) -> Result<()> {
    get_ship(container, mob_id)
        .map(|ship_id| {
            let labels = search_landing_sites(container, ship_id)
                .into_iter()
                .flat_map(|id| container.labels.get_label(id))
                .collect();

            let msg = comm::space_land_list(&labels);

            container.outputs.private(mob_id, msg);
        })
        .ok_or_else(|| {
            container
                .outputs
                .private(mob_id, comm::space_land_invalid());
            Error::InvalidArgumentFailure
        })
}

pub fn land_at(container: &mut Container, mob_id: MobId, input: &StrInput) -> Result<()> {
    get_ship(container, mob_id)
        .ok_or_else(|| Error::InvalidArgumentFailure)
        .and_then(|craft_id| {
            let sites = search_landing_sites(container, craft_id);
            let founds = container.labels.search(&sites, input.plain_arguments());

            match founds.into_iter().next() {
                Some(landing_room) => do_land_at(container, mob_id, craft_id, landing_room),
                None => Err(Error::InvalidArgumentFailure),
            }
        })
        .map_err(|err| {
            container
                .outputs
                .private(mob_id, comm::space_land_invalid());
            err
        })
}

pub fn launch(container: &mut Container, mob_id: MobId) -> Result<()> {
    get_ship(container, mob_id)
        .as_result()
        .map_err(|e| {
            container
                .outputs
                .private(mob_id, comm::space_invalid_not_in_craft());
            e
        })
        .and_then(|craft_id| do_launch(container, mob_id, craft_id))
}

pub fn jump(ctx: &mut ViewHandleCtx) -> Result<()> {
    let ship_id = get_ship(ctx.container, ctx.mob_id).as_result()?;

    do_jump(ctx.container, ctx.mob_id, ship_id)
}
