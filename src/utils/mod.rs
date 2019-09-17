use std::ops::{Sub, Add};

pub mod macros;
pub mod logs;
pub mod jsons;
pub mod save;

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct Tick(pub u32);

#[derive(Clone,Copy,Debug)]
pub struct Seconds(pub f32);

impl Seconds {
    pub fn le(&self, other: &Seconds) -> bool {
        self.0 <= other.0
    }

    pub fn ge(&self, other: &Seconds) -> bool {
        self.0 >= other.0
    }
}

impl std::ops::Add<Seconds> for Seconds {
    type Output = Seconds;

    fn add(self, rhs: Seconds) -> Seconds {
        Seconds(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Seconds> for Seconds {
    type Output = Seconds;

    fn sub(self, rhs: Seconds) -> Seconds {
        Seconds(self.0 - rhs.0)
    }
}

/// @see Trigger::check
///
#[derive(Clone,Debug)]
pub struct TimeTrigger {
    total: Seconds,
    current: Seconds,
}

pub struct TimeTriggerResult {
    pub trigger: bool,
    pub new_value: Seconds
}

impl TimeTrigger {
    pub fn new(each_second: Seconds) -> Self {
        TimeTrigger { total: each_second, current: Seconds(0.0) }
    }

    pub fn get_wait_time(&self) -> Seconds {
        self.total
    }

    pub fn get_current_time(&self) -> Seconds {
        self.current
    }

    /// Update local counter and return true if time has elapsed
    pub fn check(&mut self, elapsed: Seconds) -> bool {
        let result = TimeTrigger::check_value(elapsed, self.current, self.total);
        self.current = result.new_value;
        result.trigger
    }

    pub fn reset(&mut self) {
        self.current = Seconds(0.0);
    }

    /// execute a timer giving arguments
    pub fn check_value(elapsed: Seconds, current: Seconds, total: Seconds) -> TimeTriggerResult {
        let next = current + elapsed;
        if next.ge(&total) {
            TimeTriggerResult {
                trigger: true,
                new_value: total.sub(total)
            }
        } else {
            TimeTriggerResult {
                trigger: false,
                new_value: next
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::{TimeTrigger, Seconds};

    #[test]
    fn test_trigger() {
        let mut t = TimeTrigger::new(Seconds(1.0));
        assert_eq!(false, t.check(Seconds(0.1)));  // 0.9
        assert_eq!(false, t.check(Seconds(0.4)));  // 0.5
        assert_eq!(false, t.check(Seconds(0.39))); // 0.11
        assert_eq!(true, t.check(Seconds(0.16)));  // 1.05
        assert_eq!(true, t.check(Seconds(1.00)));  // 2.05
    }
}
