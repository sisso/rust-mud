use core::utils::{DeltaTime, UserId};

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

    pub fn add_connection(&mut self) -> UserId {
        unimplemented!()
    }

    pub fn remove_connection(&mut self, user_id: UserId) {
        unimplemented!()
    }

    pub fn take_events(&mut self) -> Vec<Output> {
        unimplemented!()
    }
}

pub enum Output {

}
