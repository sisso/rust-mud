use commons::save::{Snapshot, SnapshotSupport};
use commons::ObjId;
use logs::*;
use serde::{Deserialize, Serialize};

/// Numeric identifier of event type, used for query
#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Spawn,
    Rest,
    Combat,
    Decay,
    /// Used now for last element
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Obj { kind: EventKind, obj_id: ObjId },
}

impl Event {
    pub fn get_kind(&self) -> EventKind {
        match self {
            Event::Obj { kind, .. } => *kind,
            other => panic!("unexpected kind {:?}", other),
        }
    }

    pub fn get_obj_id(&self) -> ObjId {
        match self {
            Event::Obj { obj_id, .. } => *obj_id,
            other => panic!("unexpected kind {:?}", other),
        }
    }
}

/// Just keep a buffer for events filter by type
///
/// gc need to be called in the end of loop to clear it.
pub struct Triggers {
    index: Vec<Vec<Event>>,
}

impl Triggers {
    pub fn new() -> Self {
        let mut index = Vec::new();

        for _i in 0..EventKind::Unknown as u32 {
            index.push(Vec::new());
        }

        Triggers { index }
    }

    pub fn push(&mut self, event: Event) {
        debug!("push {:?}", event);
        self.index
            .get_mut(event.get_kind() as usize)
            .expect("wrong events initalization")
            .push(event);
    }

    pub fn list<'a>(&'a self, kind: EventKind) -> impl Iterator<Item = &Event> + 'a {
        self.index
            .get(kind as usize)
            .expect("wrong events initalization")
            .iter()
    }

    pub fn clear(&mut self) {
        for buffer in self.index.iter_mut() {
            buffer.clear();
        }
    }
}

impl SnapshotSupport for Triggers {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        use serde_json::json;
        let value = json!(self.index);
        snapshot.add_header("triggers", value);
    }

    fn load_snapshot(&mut self, _snapshot: &mut Snapshot) {
        unimplemented!()
    }
}
