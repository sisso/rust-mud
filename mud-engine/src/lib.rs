use core::utils::{DeltaTime, PlayerId};

pub struct Engine {

}

impl Engine {
    pub fn new() -> Self {
        Engine {}
    }

    pub fn load(&mut self, data_dir: &str) {

    }

    pub fn tick(&mut self, delta_time: DeltaTime) {

    }

    pub fn disconnect(&mut self, player_id: PlayerId) {
        unimplemented!()
    }

    pub fn take_events(&mut self) -> Vec<Output> {
        unimplemented!()
    }

    pub fn login(&mut self, login: &str, pass: &str) -> Result<PlayerId, ()> {
        unimplemented!()
    }
}

pub enum Output {

}
