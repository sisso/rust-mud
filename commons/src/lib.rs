use serde::{Deserialize, Serialize};

pub mod asciicolors;
pub mod csv;
pub mod grid;
pub mod io;
pub mod jsons;
pub mod prob;
pub mod repositories;
pub mod timer;
pub mod tree;
pub mod trigger;

#[macro_export]
macro_rules! unwrap_or_continue {
    ($res:expr) => {
        match $res {
            Some(value) => value,
            None => continue,
        }
    };
}

// TODO: probably don't belongs here anymore
/// Represents a connected user, not necessary is playing or have a avatar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub u32);

impl ConnectionId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

pub const MIN_DISTANCE: f32 = 0.01;
pub const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub struct V2I {
    pub x: i32,
    pub y: i32,
}

impl V2I {
    pub fn new(x: i32, y: i32) -> Self {
        V2I { x, y }
    }

    pub fn translate(&self, dx: i32, dy: i32) -> V2I {
        let new_x = self.x as i32 + dx;
        let new_y = self.y as i32 + dy;

        V2I::new(new_x, new_y)
    }

    pub fn as_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }
}

impl From<(i32, i32)> for V2I {
    fn from((x, y): (i32, i32)) -> Self {
        V2I { x: x, y: y }
    }
}

impl From<[i32; 2]> for V2I {
    fn from(array: [i32; 2]) -> Self {
        V2I {
            x: array[0],
            y: array[1],
        }
    }
}

pub struct RectI {
    topleft: V2I,
    bottomright: V2I,
}

impl RectI {
    pub fn new_2_points(topleft: V2I, bottomright: V2I) -> Self {
        RectI {
            topleft,
            bottomright,
        }
    }

    pub fn get_top_left(&self) -> V2I {
        self.topleft.clone()
    }

    pub fn get_bottom_right(&self) -> V2I {
        self.bottomright.clone()
    }

    pub fn get_width(&self) -> i32 {
        self.bottomright.x - self.topleft.x
    }

    pub fn get_height(&self) -> i32 {
        self.bottomright.y - self.topleft.y
    }

    pub fn is_inside(&self, v: &V2I) -> bool {
        v.x >= self.topleft.x
            && v.x <= self.bottomright.x
            && v.y >= self.topleft.y
            && v.y <= self.bottomright.y
    }
}

#[test]
fn test_recti_is_inside() {
    let r = RectI::new_2_points((-2, -1).into(), (2, 4).into());
    assert!(r.is_inside(&(-2, -1).into()));
    assert!(r.is_inside(&(0, 0).into()));
    assert!(r.is_inside(&(2, 4).into()));
    assert!(!r.is_inside(&(3, 4).into()));
    assert!(!r.is_inside(&(-2, -2).into()));
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
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

    pub fn distance(&self, other: V2) -> f32 {
        let delta = other.sub(self);
        let len = delta.length();
        if len.is_nan() {
            0.0
        } else {
            len
        }
    }
}

pub type Position = V2;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ObjId(pub u32);

impl ObjId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for ObjId {
    fn from(value: u32) -> Self {
        ObjId(value)
    }
}

/// Represent someone playing the game, is attached to a login, not necessary has a avatar
pub type PlayerId = ObjId;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct DeltaTime(pub f32);

impl DeltaTime {
    pub fn as_seconds_f32(&self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TotalTime(pub f64);

impl TotalTime {
    pub fn as_seconds_f64(&self) -> f64 {
        self.0 as f64
    }

    /// Or equal
    pub fn is_after(&self, time: TotalTime) -> bool {
        self.0 >= time.0
    }

    /// Or equal
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
        TotalTime(self.0 + rhs.as_seconds_f32() as f64)
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
#[derive(Clone, Debug)]
pub struct TimeTrigger {
    calm_down: DeltaTime,
    next_trigger: TotalTime,
}

impl TimeTrigger {
    pub fn new(calm_down: DeltaTime, total: TotalTime) -> Self {
        let mut t = TimeTrigger {
            calm_down,
            next_trigger: TotalTime(0.0),
        };
        t.reset(total);
        t
    }

    /// Update local counter and return true if time has elapsed
    pub fn check(&mut self, total: TotalTime) -> bool {
        match TimeTrigger::check_trigger(self.calm_down, self.next_trigger, total) {
            Some(next) => {
                self.next_trigger = next;
                true
            }
            _ => false,
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
    /// match TimeTrigger::check_trigger(EXTRACT_TIME, mob.state.extract_calm_down, container.time.total) {
    ///     Some(next) => { mob.state.extract_calm_down = next }, None => {}
    /// }
    pub fn check_trigger(
        calm_down: DeltaTime,
        next_trigger: TotalTime,
        total: TotalTime,
    ) -> Option<TotalTime> {
        if TimeTrigger::should_trigger(next_trigger, total) {
            let next = next_trigger + calm_down;
            Some(next)
        } else {
            None
        }
    }
}

pub fn vec_take<T, F>(collection: &mut Vec<T>, closure: F) -> Option<T>
where
    F: FnMut(&T) -> bool,
{
    match collection.iter().position(closure) {
        Some(index) => Some(collection.remove(index)),
        _other => None,
    }
}

pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[cfg(test)]
mod test {
    use crate::{DeltaTime, TimeTrigger, TotalTime};

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
