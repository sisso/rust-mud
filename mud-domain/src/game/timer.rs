use crate::game::triggers::{Event, Triggers};
use commons::save::{Snapshot, SnapshotSupport};
use commons::{timer::Timer as CTimer, TotalTime};
use logs::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Timer {
    index: CTimer<Event>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            index: CTimer::new(),
        }
    }

    pub fn schedule(&mut self, time: TotalTime, trigger: Event) {
        debug!("schedule {:?} at {:?}", trigger, time);
        self.index.schedule(trigger, time.as_seconds_f64());
    }

    pub fn tick(&mut self, total_time: TotalTime, triggers: &mut Triggers) {
        let events = self.index.check(total_time.as_seconds_f64());
        for event in events {
            triggers.push(event);
        }
    }
}

impl SnapshotSupport for Timer {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {
        use serde_json::json;
        let entries: Vec<&Event> = self.index.list().collect();
        let value = json!(entries);
        snapshot.add_header("timer", value);
    }
}
