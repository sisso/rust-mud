use crate::game::comm::help;

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
pub struct TimeTrigger {
    each_second: Seconds,
    next_trigger: Option<Seconds>,
}

impl TimeTrigger {
    pub fn new(each_second: Seconds) -> Self {
        TimeTrigger { each_second, next_trigger: None }
    }

    /// Update local counter and return true if time has elapsed
    pub fn check(&mut self, elapsed: Seconds) -> bool {
        match self.next_trigger {
            Some(nx) => {
                let nt = nx.0 - elapsed.0;
                if nt <= 0.0 {
                    self.next_trigger = Some(elapsed);
                    true
                } else {
                    self.next_trigger = Some(Seconds(nt));
                    false
                }
            },
            None => {
                self.next_trigger = Some(elapsed);
                false
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
        assert_eq!(false, t.check(Seconds(0.5)));  // 0.5
        assert_eq!(false, t.check(Seconds(0.49))); // 0.99
        assert_eq!(true, t.check(Seconds(0.10)));  // 1.09
        assert_eq!(true, t.check(Seconds(1.00)));  // 2.09
    }
}
