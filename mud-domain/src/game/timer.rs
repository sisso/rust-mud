use crate::game::triggers::{Event, Triggers};
use commons::{timer::Timer as CTimer, Tick, TotalTime};
use logs::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
