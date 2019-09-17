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
    pub fn le(&self, other: Seconds) -> bool {
        self.0 <= other.0
    }

    pub fn ge(&self, other: Seconds) -> bool {
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
    calm_down: Seconds,
    next_trigger: Seconds,
}

impl TimeTrigger {
    pub fn new(calm_down: Seconds, total: Seconds) -> Self {
        let mut t = TimeTrigger { calm_down, next_trigger: Seconds(0.0) };
        t.reset(total);
        t
    }

    /// Update local counter and return true if time has elapsed
    pub fn check(&mut self, total: Seconds) -> bool {
        match TimeTrigger::check_trigger(self.calm_down, self.next_trigger, total) {
            Some(next) => {
                self.next_trigger = next;
                true
            },
            _ => false
        }
    }

    pub fn reset(&mut self, total: Seconds) {
        self.next_trigger = total.add(self.calm_down);
    }

    pub fn next(next_trigger: Seconds, total: Seconds) -> Seconds {
        total + next_trigger
    }

    pub fn should_trigger(total: Seconds, next_trigger: Seconds) -> bool {
        total.ge(next_trigger)
    }

    /// If trigger, return next trigger
    pub fn check_trigger(calm_down: Seconds, next_trigger: Seconds, total: Seconds) -> Option<Seconds> {
        if TimeTrigger::should_trigger(total, next_trigger) {
            let next = total - next_trigger + calm_down;
            Some(next)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::{TimeTrigger, Seconds};

    #[test]
    fn test_trigger() {
        let mut t = TimeTrigger::new(Seconds(1.0), Seconds(0.0));
        assert_eq!(false, t.check(Seconds(0.1)));
        assert_eq!(false, t.check(Seconds(0.2)));
        assert_eq!(false, t.check(Seconds(0.99)));
        assert_eq!(true, t.check(Seconds(1.01)));
        assert_eq!(true, t.check(Seconds(2.02)));
    }
}
