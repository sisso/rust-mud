use crate::errors::*;
use crate::game::ai::{Ai, AiCommand};
use crate::game::container::Container;
use crate::game::location::LocationId;
use crate::game::mob::MobRepository;
use commons::ObjId;

pub enum RequestCommand {
    Idle,
    FollowMe,
    Extract,
}

pub fn list_commandable(container: &Container, obj_id: ObjId) -> Result<Vec<ObjId>> {
    let l = container
        .ownership
        .list(obj_id)
        .into_iter()
        .filter(|id| is_commandable(container, *id))
        .collect();
    Ok(l)
}

fn is_commandable(container: &Container, id: ObjId) -> bool {
    container
        .ai
        .get(id)
        .map(|ai| ai.commandable)
        .unwrap_or(false)
}

pub fn find_commandable(container: &Container, obj_id: ObjId, label: &str) -> Result<Vec<ObjId>> {
    let ids = list_commandable(container, obj_id)?;
    Ok(container.labels.search(&ids, label))
}

pub fn list_commands_for(_container: &Container, _obj_id: ObjId) -> Result<Vec<RequestCommand>> {
    Ok(vec![
        RequestCommand::Idle,
        RequestCommand::FollowMe,
        RequestCommand::Extract,
    ])
}

pub fn set_command_follow(
    container: &mut Container,
    obj_id: ObjId,
    followed_id: ObjId,
) -> Result<()> {
    let ai = container
        .ai
        .get_mut(obj_id)
        .as_result_str("target object expected to have AI")?;

    clear_ai_command(ai, &mut container.mobs)?;

    ai.command = AiCommand::FollowAndProtect {
        target_id: followed_id,
    };

    container.mobs.add_follower(followed_id, obj_id)?;

    Ok(())
}

pub fn set_command_extract(
    container: &mut Container,
    mob_id: ObjId,
    location_id: LocationId,
    extractable_id: ObjId,
) -> Result<()> {
    let ai = container
        .ai
        .get_mut(mob_id)
        .as_result_str("target object expected to have AI")?;

    clear_ai_command(ai, &mut container.mobs)?;

    // clear previous command
    ai.command = AiCommand::Extract {
        from: extractable_id,
    };

    super::actions::extract(container, mob_id, location_id, extractable_id)?;

    Ok(())
}

fn clear_ai_command(ai: &mut Ai, mobs: &mut MobRepository) -> Result<()> {
    match ai.command {
        AiCommand::FollowAndProtect { target_id } => {
            mobs.remove_follower(target_id, ai.id)?;
        }
        _ => {}
    }

    ai.command = AiCommand::Idle;

    Ok(())
}
