pub mod macros;

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


