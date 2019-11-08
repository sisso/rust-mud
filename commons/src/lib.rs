pub mod jsons;
pub mod save;


#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct ConnectionId(pub u32);

impl ConnectionId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

pub const MIN_DISTANCE: f32 = 0.01;
pub const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

#[derive(Clone,Copy,PartialEq,Debug)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl V2 {
    pub fn new(x: f32, y: f32) -> Self {
        V2 { x, y }
    }

    pub fn add(&self, other: &V2) -> V2 {
        V2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn sub(&self, other: &V2) -> V2 {
        V2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn normalized(&self) -> V2 {
        self.mult(1.0 / self.length_sqr().sqrt())
    }

    pub fn length(&self) -> f32 {
        self.length_sqr().sqrt()
    }

    pub fn length_sqr(&self) -> f32 {
        (self.x * self.x) + (self.y * self.y)
    }

    pub fn mult(&self, scale: f32) -> V2 {
        V2 {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    pub fn div(&self, scale: f32) -> V2 {
        self.mult(1.0 / scale)
    }
}

pub type Position = V2;

#[derive(Clone,Copy,Debug)]
pub struct Tick(pub u32);

impl Tick {
    pub fn new() -> Self {
        Tick(0)
    }

    pub fn next(&self) -> Tick {
        Tick(self.0 + 1)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub struct ObjId(pub u32);

impl ObjId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

pub type PlayerId = ObjId;

#[derive(Clone,Copy,Debug)]
pub struct DeltaTime(pub f32);

impl DeltaTime{
    pub fn as_second(&self) -> DeltaTime {
        DeltaTime(self.0)
    }

    pub fn as_f32(&self) -> f32 { self.0 }

    pub fn as_f64(&self) -> f64 { self.0 as f64 }
}

#[derive(Clone,Copy,Debug)]
pub struct TotalTime(pub f64);

impl TotalTime{
    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn is_after(&self, time: TotalTime) -> bool {
        self.0 >= time.0
    }

    pub fn is_before(&self, time: TotalTime) -> bool {
        self.0 <= time.0
    }

    pub fn add(&self, delta: DeltaTime) -> TotalTime {
        TotalTime(self.0 + delta.0 as f64)
    }

    pub fn sub(&self, other: TotalTime) -> DeltaTime {
        DeltaTime((self.0 - other.0) as f32)
    }
}

impl std::ops::Add<DeltaTime> for TotalTime {
    type Output = TotalTime;

    fn add(self, rhs: DeltaTime) -> TotalTime {
        TotalTime(self.0 + rhs.as_f64())
    }
}

impl std::ops::Sub<DeltaTime> for DeltaTime {
    type Output = DeltaTime;

    fn sub(self, rhs: DeltaTime) -> DeltaTime {
        DeltaTime(self.0 - rhs.0)
    }
}

/// @see Trigger::check
///
#[derive(Clone,Debug)]
pub struct TimeTrigger {
    calm_down: DeltaTime,
    next_trigger: TotalTime,
}

impl TimeTrigger {
    pub fn new(calm_down: DeltaTime, total: TotalTime) -> Self {
        let mut t = TimeTrigger { calm_down, next_trigger: TotalTime(0.0) };
        t.reset(total);
        t
    }

    /// Update local counter and return true if time has elapsed
    pub fn check(&mut self, total: TotalTime) -> bool {
        match TimeTrigger::check_trigger(self.calm_down, self.next_trigger, total) {
            Some(next) => {
                self.next_trigger = next;
                true
            },
            _ => false
        }
    }

    pub fn reset(&mut self, total: TotalTime) {
        self.next_trigger = total + self.calm_down;
    }

    pub fn next(next_trigger: DeltaTime, total: TotalTime) -> TotalTime {
        total + next_trigger
    }

    pub fn should_trigger(next_trigger: TotalTime, total: TotalTime) -> bool {
        total.is_after(next_trigger)
    }

    /// If trigger, return next trigger
    pub fn check_trigger(calm_down: DeltaTime, next_trigger: TotalTime, total: TotalTime) -> Option<TotalTime> {
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
    use crate::{TimeTrigger, DeltaTime, TotalTime};

    #[test]
    fn test_trigger() {
        let mut t = TimeTrigger::new(DeltaTime(1.0), TotalTime(0.0));
        assert_eq!(false, t.check(TotalTime(0.1)));
        assert_eq!(false, t.check(TotalTime(0.2)));
        assert_eq!(false, t.check(TotalTime(0.99)));
        assert_eq!(true, t.check(TotalTime(1.01)));
        assert_eq!(true, t.check(TotalTime(2.02)));
    }
}
