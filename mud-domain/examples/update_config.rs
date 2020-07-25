use serde_json::{json, Value};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type UpdateValue = fn(&mut Value);

trait FieldUpdater {
    fn update(&mut self, value: &mut Value);
}

struct AppendUpdater {
    field: String,
    input: Value,
}

impl FieldUpdater for AppendUpdater {
    fn update(&mut self, value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.insert(self.field.clone(), self.input.clone());
            }

            other => {
                panic!("unexpected {:?}", other);
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let path = args[1].as_str();

    let file = std::fs::read_to_string(path)?;
    let mut json = serde_json::from_str(file.as_str())?;

    // append_field(&mut json, "tag", "[]")?;
    let mut change = AppendUpdater {
        field: "tag".to_string(),
        input: json!([]),
    };

    update_each_object(&mut json, &mut change);

    println!("{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}

fn update_each_object(root: &mut Value, mut updater: &mut dyn FieldUpdater) {
    if let Some(object_list) = root["objects"].as_object_mut() {
        for (_, v) in object_list {
            // println!("{:?}", v);
            updater.update(v);
        }
    } else {
        panic!("objects is not a object: {:?}", root["objects"]);
    }
}

fn show_keys(value: &Value) {
    if let Some(obj) = value.as_object() {
        for k in obj.keys() {
            println!("{:?}", k);
        }
    } else {
        panic!("object is not a map {:?}", value);
    }
}
