use crate::game::container::Container;
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::utils::strinput::StrInput;
use crate::errors::{Result, Error, AsResult};
use logs::*;
use crate::errors::Error::NotFoundFailure;
use crate::game::actions::out;

pub fn hire(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    hired_id: MobId,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        warn!("{:?} player has no location", mob_id);
        outputs.private(mob_id, comm::hire_fail());
        Error::InvalidStateFailure
    })?;

    let mob = container.mobs.get_mut(mob_id).as_result()?;
    mob.followers.push(hired_id);

    container.ownership.set_owner(hired_id, mob_id);

    let mob_label = container.labels.get_label_f(mob_id);
    let hired_label = container.labels.get_label_f(hired_id);

    outputs.private(mob_id, comm::hire(hired_label));
    outputs.broadcast(Some(mob_id), location_id, comm::hire_others(mob_label, hired_label));

    Ok(())
}
