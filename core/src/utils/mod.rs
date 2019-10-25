pub mod jsons;
pub mod save;

// TODO: to DeltaTime and TotalTime
#[derive(Clone,Copy,Debug)]
pub struct Second(pub f32);

#[derive(Clone,Copy,Debug)]
pub struct DeltaTime(pub f32);

#[derive(Clone,Copy,Debug)]
pub struct TotalTime(pub f64);

///
/// User connected to the engine, not necessary logged
///
#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct UserId(pub u32);


impl Second {
    pub fn one() -> Second {
        Second(1.0)
    }
}

impl Second {
    pub fn le(&self, other: Second) -> bool {
        self.0 <= other.0
    }

    pub fn ge(&self, other: Second) -> bool {
        self.0 >= other.0
    }

    pub fn as_float(&self) -> f32 {
        self.0
    }
}

impl std::ops::Add<Second> for Second {
    type Output = Second;

    fn add(self, rhs: Second) -> Second {
        Second(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Second> for Second {
    type Output = Second;

    fn sub(self, rhs: Second) -> Second {
        Second(self.0 - rhs.0)
    }
}

/// @see Trigger::check
///
#[derive(Clone,Debug)]
pub struct TimeTrigger {
    calm_down: Second,
    next_trigger: Second,
}

impl TimeTrigger {
    pub fn new(calm_down: Second, total: Second) -> Self {
        let mut t = TimeTrigger { calm_down, next_trigger: Second(0.0) };
        t.reset(total);
        t
    }

    /// Update local counter and return true if time has elapsed
    pub fn check(&mut self, total: Second) -> bool {
        match TimeTrigger::check_trigger(self.calm_down, self.next_trigger, total) {
            Some(next) => {
                self.next_trigger = next;
                true
            },
            _ => false
        }
    }

    pub fn reset(&mut self, total: Second) {
        self.next_trigger = total + self.calm_down;
    }

    pub fn next(next_trigger: Second, total: Second) -> Second {
        total + next_trigger
    }

    pub fn should_trigger(next_trigger: Second, total: Second) -> bool {
        total.ge(next_trigger)
    }

    /// If trigger, return next trigger
    pub fn check_trigger(calm_down: Second, next_trigger: Second, total: Second) -> Option<Second> {
        if TimeTrigger::should_trigger(next_trigger, total) {
            let next = next_trigger + calm_down;
            Some(next)
        } else {
            None
        }
    }
}

pub fn vec_take<T, F>(collection: &mut Vec<T>, closure: F) -> Option<T>
    where F: FnMut(&T) -> bool {
    match collection.iter().position(closure) {
        Some(index) => Some(collection.remove(index)),
        other => None,
    }
}

#[cfg(test)]
mod test {
    use crate::utils::{TimeTrigger, Second};

    #[test]
    fn test_trigger() {
        let mut t = TimeTrigger::new(Second(1.0), Second(0.0));
        assert_eq!(false, t.check(Second(0.1)));
        assert_eq!(false, t.check(Second(0.2)));
        assert_eq!(false, t.check(Second(0.99)));
        assert_eq!(true, t.check(Second(1.01)));
        assert_eq!(true, t.check(Second(2.02)));
    }
}
