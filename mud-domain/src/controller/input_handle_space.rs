use crate::game::actions_ships;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::game::space_utils::*;
use crate::game::{comm, Outputs};
use crate::utils::text;
use commons::{PlayerId, ObjId};
use logs::*;
use crate::errors::{AsResult, Error, Result};
use crate::game::actions_ships::{do_land_at, do_launch, do_jump};
use crate::game::obj::Obj;
use crate::game::comm::ShowSectorTreeBody;
use crate::utils::strinput::StrInput;
use crate::game::astro_bodies::AstroBody;
use crate::controller::ViewHandleCtx;

pub fn show_startree(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<()> {
    let (ship_id, sector_id) = get_ship_and_sector(container, outputs, mob_id)?;
    let bodies = find_showsector_bodies(container, sector_id, Some(ship_id));
    trace!("{:?} at {:?} on sector {:?} can view {:?}", mob_id, ship_id, sector_id, bodies);
    outputs.private(mob_id, comm::show_sectortree(sector_id, &bodies));
    Ok(())
}

#[deprecated]
pub fn show_surface_map(container: &Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    let (craft_id, sector_id) = get_ship_and_sector(container, outputs, mob_id)?;
    let objects = get_objects_in_surface(container, craft_id, sector_id);
    outputs.private(mob_id, comm::show_surface_map(&objects));
    Ok(())
}

pub fn move_list_targets(
    container: &Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
) -> Result<()> {
    let (ship_id, sector_id) = get_ship_and_sector(container, outputs, mob_id)?;
    let targets = find_move_targets(container, sector_id, ship_id)
        .into_iter()
        .map(|body| to_showsectortreebody(container, Some(ship_id), body))
        .collect();
    outputs.private(mob_id, comm::space_show_move_targets(&targets));
    Ok(())
}

/// Search for space bodies that can be used to "move"
fn find_move_targets(container: &Container, sector_id: ObjId, ship_id: ObjId) -> Vec<&AstroBody> {
    find_space_bodies(container, sector_id)
        .into_iter()
        .filter(|i| i.id != ship_id)
        .collect()
}

pub fn move_to(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: &StrInput,
) -> Result<()> {
    let (ship_id, sector_id) = get_ship_and_sector(container, outputs, mob_id)?;
    let targets = find_move_targets(container, sector_id, ship_id)
        .into_iter()
        .map(|i| i.id)
        .collect();

    let founds = container.labels.search(&targets, input.plain_arguments());

    match founds.get(0) {
        Some(&target_id) =>
             actions_ships::move_to(container, outputs, mob_id, ship_id, target_id),

        None => {
            outputs.private(mob_id, comm::space_move_invalid());
            Err(Error::InvalidArgumentFailure)
        },
    }
}

pub fn land_list(container: &Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    get_ship(container, mob_id)
        .map(|ship_id| {
            let labels = search_landing_sites(container, ship_id)
                .into_iter()
                .flat_map(|id| container.labels.get_label(id))
                .collect();

            outputs.private(mob_id, comm::space_land_list(&labels));
        })
        .ok_or_else(|| {
            outputs.private(mob_id, comm::space_land_invalid());
            Error::InvalidArgumentFailure
        })
}

pub fn land_at(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: Vec<&str>,
) -> Result<()> {
    let result = match (input.get(1), get_ship(container, mob_id)) {
        (Some(input), Some(craft_id)) => {
            let sites = search_landing_sites(container, craft_id);
            let labels = container.labels.resolve_codes(&sites);

            match text::search_label(input, &labels).first().cloned() {
                Some(index) => {
                    let landing_room = sites[index];
                    do_land_at(container, outputs, craft_id, landing_room)
                }
                None => Err(Error::InvalidArgumentFailure),
            }
        }
        _ => Err(Error::InvalidArgumentFailure),
    };

    result.map_err(|e| {
        outputs.private(mob_id, comm::space_land_invalid());
        e
    })
}

pub fn launch(container: &mut Container, outputs: &mut dyn Outputs, mob_id: MobId) -> Result<()> {
    get_ship(container, mob_id)
        .as_result()
        .map_err(|e| {
            outputs.private(mob_id, comm::space_invalid_not_in_craft());
            e
        })
        .and_then(|craft_id| do_launch(container, outputs, mob_id, craft_id))
}

pub fn jump(ctx: &mut ViewHandleCtx) -> Result<()> {
    let ship_id = get_ship(ctx.container, ctx.mob_id)
        .as_result()?;

    do_jump(ctx.container, ctx.outputs, ctx.mob_id, ship_id)
}

