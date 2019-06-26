use super::domain::*;

pub struct SpawnDelay {
    pub min: Seconds,
    pub max: Seconds
}

pub struct SpawnPrefab {
    pub probability_0_100: u32,
    pub prefab_id: MobPrefabId
}

pub struct Spawn {
    pub rooms: Vec<RoomId>,
    pub max: u32,
    pub delay: SpawnDelay,
    pub prefabs: Vec<SpawnPrefab>
}
