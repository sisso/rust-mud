use crate::errors::Error::NotFoundFailure;
use crate::errors::{AsResult, Error, Result};
use crate::game::actions::out;
use crate::game::container::Container;
use crate::game::mob::MobId;
use crate::game::{comm, outputs::Outputs};
use crate::utils::strinput::StrInput;

pub fn hire(container: &mut Container, mob_id: MobId, hired_id: MobId) -> Result<()> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        log::warn!("{:?} player has no location", mob_id);
        container.outputs.private(mob_id, comm::hire_fail());
        Error::InvalidStateFailure
    })?;

    let mob = container.mobs.get_mut(mob_id).as_result()?;
    mob.followers.push(hired_id);

    container.ownership.set_owner(hired_id, mob_id);

    let mob_label = container.labels.get_label_f(mob_id);
    let hired_label = container.labels.get_label_f(hired_id);

    container.outputs.private(mob_id, comm::hire(hired_label));
    container.outputs.broadcast(
        Some(mob_id),
        location_id,
        comm::hire_others(mob_label, hired_label),
    );

    Ok(())
}
