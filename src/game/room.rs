use super::domain::Dir;
use crate::game::item::Inventory;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct RoomId(pub u32);

#[derive(Clone, Debug)]
pub struct Room {
    pub id: RoomId,
    pub label: String,
    pub desc: String,
    pub exits: Vec<(Dir, RoomId)>,
    pub inventory: Inventory,
}

impl Room {
}

