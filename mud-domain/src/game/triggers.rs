use commons::{
    ObjId,
    trigger::Trigger as CTrigger
};
use commons::trigger::Listener;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Kind {
    Spawn,
    Rest, 
    Combat,
    Decay,
}

#[derive(Debug, Clone)]
pub enum Event {
    Obj { 
        kind: Kind,
        obj_id: ObjId
    }
}

impl Event {
    pub fn get_kind(&self) -> Kind {
        match self {
            Event::Obj { kind, .. } => *kind,
            other => panic!("unexpected")
        }
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

    pub fn registre(&mut self, kind: Kind) -> Listener {
        self.index.register(kind as u32)
    }

    pub fn emit(&mut self, event: Event) {
        self.index.push(event.get_kind() as u32, event);
    }

    pub fn take(&mut self, listener: Listener) -> Vec<&Event> {
        self.take(listener)
    }

    pub fn gc(&mut self) {
        self.index.gc();
    }
}
