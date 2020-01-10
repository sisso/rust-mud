use commons::{
    TotalTime,
    timer::{Timer as CTimer}
};
use super::trigger::TriggerEvent;
use logs::*;

pub struct Timer {
    index: CTimer<TriggerEvent>
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            index: CTimer::new(),
        }
    }
}

//pub fn run(total_time: TotalTime, timer: &mut Timer, trigger: &mut Trigger) -> Result<()> {
//
//}
