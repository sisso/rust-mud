use crate::game::container::Container;
use crate::game::{Outputs, comm};
use crate::game::mob::MobId;
use crate::utils::strinput::StrInput;
use crate::errors::{Result, Error};
use logs::*;
use crate::errors::Error::NotFoundFailure;
use crate::game::actions::out;

pub fn hire(
    container: &mut Container,
    outputs: &mut dyn Outputs,
    mob_id: MobId,
    input: StrInput,
) -> Result<()> {
    let location_id = container.locations.get(mob_id).ok_or_else(|| {
        warn!("{:?} player has no location", mob_id);
        outputs.private(mob_id, comm::hire_fail());
        Error::InvalidStateFailure
    })?;

    let candidates = container.locations.list_at(location_id)
        .filter(|obj_id| container.hires.exist(*obj_id))
        .collect::<Vec<_>>();

    let args = input.plain_arguments();
    let founds = container.labels.search_codes(&candidates, args);

    if let Some(&hired_id) = founds.first() {
        let mob_label = container.labels.get_label_f(mob_id);
        let hired_label = container.labels.get_label_f(hired_id);

        container.ownership.set_owner(hired_id, mob_id);

        outputs.private(mob_id, comm::hire(hired_label));
        outputs.broadcast(Some(mob_id), location_id, comm::hire_others(mob_label, hired_label));

        Ok(())
    } else {
        let labels = container.labels.resolve_labels(&candidates);

        if args.is_empty() {
            outputs.private(mob_id, comm::hire_list(labels));
            Ok(())
        } else {
            outputs.private(mob_id, comm::hire_fail_not_found(args));
            outputs.private(mob_id, comm::hire_list(labels));
            Err(NotFoundFailure)
        }
    }
}

