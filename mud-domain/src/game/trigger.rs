use commons::{
    ObjId,
    trigger::Trigger as CTrigger
};

pub enum Kind {
    Spawn,
    Rest, 
    Combat,
    Decay,
}

pub enum Event {
    Obj { 
        kind: Kind, 
        obj_id: ObjId 
    }
}

pub struct Triggers {
    index: CTrigger<Event>,
}

impl Triggers {
    pub fn new() -> Self {
        Triggers {
            index: CTrigger::new(),
        }
    }

    pub fn registrer(&mut self, kind: TriggerKind) -> Listener {
        unimplemented!();
    }

    pub fn emit(&mut self, event: Event) {
        self.index.push(event.kind, event);
    }

    pub fn take(&mut self, listener: Listener) -> Vec<&Event> {
        self.take(listener)
    }
}
