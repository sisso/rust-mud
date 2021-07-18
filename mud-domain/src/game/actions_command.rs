use crate::errors::*;
use crate::game::ai::Ai;
use crate::game::container::Container;
use commons::ObjId;

pub fn list_commandable(container: &Container, obj_id: ObjId) -> Result<Vec<&Ai>> {
    let l = container
        .ownership
        .list(obj_id)
        .iter()
        .flat_map(|id| container.ai.get(*id))
        .collect();
    Ok(l)
}
