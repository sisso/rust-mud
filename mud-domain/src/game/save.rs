use crate::errors::Result;
use crate::game::container::Container;
use crate::game::mob::Mob;
use crate::game::player::Player;

// TODO: continue
pub fn save(container: &Container) -> Result<Save> {
    let mut save = Save::new();

    for player in container.players.list() {
        save.save_player(player)?;

        if let Some(avatar) = container.mobs.get(player.mob_id) {
            let location_id = container.locations.get(avatar.id);
            save.save_mob(avatar);

            for id in container.locations.list_deep_at(avatar_id) {}
        }
    }

    Ok(save)
}

pub struct Save {}

impl Save {
    pub fn new() -> Self {
        Save {}
    }

    pub fn save_player(&mut self, player: &Player) -> Result<()> {
        unimplemented!()
    }

    pub fn save_mob(&mut self, mob: &Mob) -> Result<()> {
        unimplemented!()
    }
}
