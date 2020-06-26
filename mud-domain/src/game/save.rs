use crate::errors::Error;
use crate::errors::Result;
use crate::game::container::Container;
use crate::game::snapshot::{Snapshot, SnapshotSupport};
use logs::*;

pub fn save_to_file(container: &Container, path_and_file_prefix: &str) -> Result<()> {
    info!("saving at {}", path_and_file_prefix);

    let mut snapshot = Snapshot::new();
    container.save_snapshot(&mut snapshot);
    snapshot.save_to_file(format!("{}.save", path_and_file_prefix).as_str());
    snapshot.save_to_file(
        format!(
            "{}_{}.snapshot",
            path_and_file_prefix,
            container.time.tick.as_u32()
        )
        .as_str(),
    );
    Ok(())
}

// TODO: since it only use container, should not be part of the Game
// TODO: load can only happens considering load module
pub fn load_from_file(container: &mut Container, path_and_file_prefix: &str) -> Result<()> {
    let snapshot =
        Snapshot::load(format!("{}.save", path_and_file_prefix).as_str()).map_err(|e| match e {
            crate::game::snapshot::Error::FileNotFound { path } => {
                info!("skipping loading, profile file not found at {}", path);
                Error::NotFoundFailure
            }
            other => Error::Exception(format!("{:?}", other)),
        })?;
    container.load_snapshot(&snapshot);
    Ok(())
}

// use crate::errors::Result;
// use crate::game::container::Container;
// use crate::game::loader::StaticId;
// use crate::game::mob::Mob;
// use crate::game::player::Player;
// use commons::ObjId;
// use serde::{Deserialize, Serialize};
//
// // TODO: remove
// pub fn save(container: &Container) -> Result<Save> {
//     // let mut save = Save::new();
//     //
//     // for player in container.players.list() {
//     //     let avatar_save = if let Some(avatar) = container.mobs.get(player.mob_id) {
//     //         // let location_id = container.locations.get(avatar.id);
//     //         //
//     //         // let avatar_static_id = container.objects.get_static_id(avatar.id).unwrap();
//     //         // save.save_mob(location_id, avatar_static_id, avatar);
//     //         //
//     //         // for id in container.locations.list_deep_at(avatar_id) {
//     //         //
//     //         // }
//     //         Some(AvatarSave {})
//     //     } else {
//     //         None
//     //     };
//     //
//     //     save.players.push(PlayerSave {
//     //         label: player.login.clone(),
//     //         avatar: avatar_save,
//     //     });
//     // }
//     //
//     // Ok(save)
//     unimplemented!()
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ItemSave {
//     static_id: StaticId,
//     amount: u32,
//     equiped: bool,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct AvatarSave {
//     static_id: StaticId,
//     location_id: ObjId,
//     items: Vec<ItemSave>,
//     xp: u32,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct PlayerSave {
//     label: String,
//     avatar: Option<AvatarSave>,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Save {
//     pub players: Vec<PlayerSave>,
// }
//
// impl Save {
//     pub fn new() -> Self {
//         Save {
//             players: Default::default(),
//         }
//     }
//
//     pub fn save_player(&mut self, player: &Player) -> Result<()> {
//         unimplemented!()
//     }
//
//     pub fn save_mob(&mut self, static_id: StaticId, mob: &Mob) -> Result<()> {
//         unimplemented!()
//     }
// }
