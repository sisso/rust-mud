use commons::{
    ObjId,
    trigger::Trigger as CTrigger
};

pub enum TriggerEvent {
    Spawn { obj_id: ObjId },
    Rest { obj_id: ObjId },
    Combat { obj_id: ObjId },
    Decay { obj_id: ObjId },
}

pub struct Trigger {
    index: CTrigger<TriggerEvent>,
}

impl Trigger {
    pub fn new() -> Self {
        Trigger {
            index: CTrigger::new(),
        }
    }
}
