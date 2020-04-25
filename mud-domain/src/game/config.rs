use crate::game::loader::StaticId;
use crate::game::room::RoomId;
use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub initial_room: Option<RoomId>,
    pub avatar_id: Option<StaticId>,
    pub money_id: Option<StaticId>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            initial_room: None,
            avatar_id: None,
            money_id: None,
        }
    }
}

impl SnapshotSupport for Config {
    fn save(&self, snapshot: &mut Snapshot) {
        use serde_json::json;
        snapshot.add_header("config", json!(self));
    }

    fn load(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}
