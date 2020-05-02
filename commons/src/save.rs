use serde_json::{json, Value};

use std::fs::File;

use std::collections::HashMap;
use std::io::{BufRead, Write};

/*

{ header: $key, [$property: $value]* }
{ id: $id, [$property: $value]* }

*/
pub trait SnapshotSupport {
    fn save_snapshot(&self, snapshot: &mut Snapshot) {}
    fn load_snapshot(&mut self, snapshot: &mut Snapshot) {}
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    version: u32,
    headers: HashMap<String, Value>,
    objects: HashMap<u32, HashMap<String, Value>>,
}

impl Snapshot {
    pub fn new() -> Self {
        Snapshot {
            version: 0,
            headers: Default::default(),
            objects: Default::default(),
        }
    }

    pub fn add_header(&mut self, header_name: &str, header_value: serde_json::Value) {
        self.headers.insert(header_name.to_string(), header_value);
    }

    pub fn add(&mut self, id: u32, component: &str, value: serde_json::Value) {
        let m = self.objects.entry(id).or_insert_with(|| {
            let mut m = HashMap::new();
            m.insert("id".to_string(), json!(id));
            m
        });

        m.insert(component.to_string(), value);
    }

    pub fn get_headers(&self, header: &str) -> Option<&Value> {
        self.headers.get(header)
    }

    pub fn get_components(&self, component: &str) -> Vec<(u32, &Value)> {
        self.objects
            .iter()
            .flat_map(|(id, value)| match value.get(component) {
                Some(value) => Some((*id, value)),
                None => None,
            })
            .collect()
    }

    pub fn save_to_file(&self, file_path: &str) {
        let mut file = File::create(file_path.to_string()).unwrap();

        for (header, value) in &self.headers {
            let json = json!({
                "header": header,
                "value": value,
            });
            file.write(json.to_string().as_bytes()).unwrap();
            file.write("\n".as_bytes()).unwrap();
        }

        for (_id, components) in &self.objects {
            let json = json!(components);
            file.write(json.to_string().as_bytes()).unwrap();
            file.write("\n".as_bytes()).unwrap();
        }

        file.flush().unwrap();
    }

    pub fn load(file_path: &str) -> Self {
        let file = File::open(file_path).expect(&format!("failed to open file {:?}", file_path));

        let lines = std::io::BufReader::new(file).lines();

        let mut save = Snapshot::new();
        for line in lines {
            let line = line.unwrap();
            let mut value: HashMap<String, Value> = serde_json::from_str(line.as_str()).unwrap();

            if let Some(header) = value.remove("header") {
                let value = value.remove("value").unwrap();
                let header: String = match header {
                    Value::String(str) => str,
                    _ => panic!(),
                };
                save.headers.insert(header, value);
            } else {
                let id: u32 = value.get("id").unwrap().as_i64().unwrap() as u32;
                save.objects.insert(id, value);
            }
        }

        save
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    pub fn save_and_load_test() {
        let file_path = "/tmp/test.save";
        let _ = fs::remove_file(file_path);

        {
            let mut save = Snapshot::new();
            save.add_header("start_room", json!(22));
            save.add(
                0,
                "label",
                json!({
                    "label": "Room 1",
                    "desc": "No description"
                }),
            );
            save.add(
                0,
                "room",
                json!({
                    "exits": [
                        { "dir": "n", "id": 1 }
                    ]
                }),
            );
            save.add(
                1,
                "label",
                json!({
                    "label": "Room 2",
                    "desc": "No description"
                }),
            );
            save.add(
                0,
                "label",
                json!({
                    "label": "Room 1",
                }),
            );

            save.save_to_file(file_path);
        }

        {
            let load = Snapshot::load(file_path);
            let labels = load.get_components("label");

            assert_eq!(
                22 as i64,
                load.get_headers("start_room").unwrap().as_i64().unwrap()
            );

            assert_eq!(2, labels.len());
            match labels.first() {
                Some((id, value)) if *id == 0 => {
                    let value = value["label"].as_str().unwrap();
                    assert_eq!("Room 1", value);
                }
                Some((id, value)) if *id == 1 => {
                    let value = value["label"].as_str().unwrap();
                    assert_eq!("Room 2", value);
                }
                _ => panic!(),
            }
        }
    }
}
