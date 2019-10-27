use serde_json::Value;
use serde_json::json;

pub trait JsonValueExtra {
    fn to_f32(&self) -> f32;
    fn to_u32(&self) -> u32;
    fn as_opt(&self) -> Option<&Value>;
}

impl JsonValueExtra for Value {
    fn to_f32(&self) -> f32 {
        self.as_f64().unwrap() as f32
    }

    fn to_u32(&self) -> u32 {
        self.as_u64().unwrap() as u32
    }

    fn as_opt(&self) -> Option<&Value> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }
}
