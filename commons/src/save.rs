use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::fs::File;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};

/*

TODO: update code to the new format

New format

{ header: $key, [$property: $value]* }
{ id: $id, [$property: $value]* }

OLD Format

{
  headers: {
    $key: $Value
  }

  objects: {
     $id: {
        $key: $value
     }
  }
}

*/

pub trait CanSave {
    fn save(&self, save: &mut impl Save);
}

pub trait CanLoad {
    fn load(&mut self, load: &mut impl Load);
}

pub trait Save {
    fn add_header(&mut self, header_name: &str, value: serde_json::Value);
    fn add(&mut self, id: u32, component: &str, value: serde_json::Value);
}

pub trait Load {
    fn get_headers(&self, header: &str) -> Option<&serde_json::Value>;
    fn get_components(&self, component: &str) -> Vec<(u32, &serde_json::Value)>;
}

pub struct SaveToFile {
    file_path: String,
    raw: RawData,
}

#[derive(Debug, Clone)]
struct RawData {
    headers: HashMap<String, Value>,
    objects: HashMap<u32, HashMap<String, Value>>,
}

impl RawData {
    pub fn new() -> Self {
        RawData {
            headers: Default::default(),
            objects: Default::default(),
        }
    }
}

impl SaveToFile {
    pub fn new(file_path: &str) -> Self {
        SaveToFile {
            file_path: file_path.to_string(),
            raw: RawData::new(),
        }
    }
}

impl Save for SaveToFile {
    fn add_header(&mut self, header_name: &str, header_value: serde_json::Value) {
        self.raw
            .headers
            .insert(header_name.to_string(), header_value);
    }

    fn add(&mut self, id: u32, component: &str, value: serde_json::Value) {
        let m = self.raw.objects.entry(id).or_insert_with(|| {
            let mut m = HashMap::new();
            m.insert("id".to_string(), json!(id));
            m
        });

        m.insert(component.to_string(), value);
    }
}

impl Drop for SaveToFile {
    fn drop(&mut self) {
        let mut file = File::create(self.file_path.to_string()).unwrap();

        for (header, value) in &self.raw.headers {
            let json = json!({
                "header": header,
                "value": value,
            });
            file.write(json.to_string().as_bytes()).unwrap();
            file.write("\n".as_bytes());
        }

        for (id, components) in &self.raw.objects {
            let json = json!(components);
            file.write(json.to_string().as_bytes()).unwrap();
            file.write("\n".as_bytes());
        }

        // let json_data = json!(self.raw).to_string();
        // file.write_all(json_data.as_str().as_bytes()).unwrap();
        file.flush().unwrap();
    }
}

#[derive(Debug)]
pub struct LoadFromFile {
    raw: RawData,
}

impl LoadFromFile {
    pub fn new(file_path: &str) -> Self {
        let mut file =
            File::open(file_path).expect(&format!("failed to open file {:?}", file_path));

        let lines = std::io::BufReader::new(file).lines();

        let mut raw = RawData::new();
        for line in lines {
            let line = line.unwrap();
            let mut value: HashMap<String, Value> = serde_json::from_str(line.as_str()).unwrap();

            if let Some(header) = value.remove("header") {
                let value = value.remove("value").unwrap();
                let header: String = match header {
                    Value::String(str) => str,
                    _ => panic!(),
                };
                raw.headers.insert(header, value);
            } else {
                let id: u32 = value.get("id").unwrap().as_i64().unwrap() as u32;
                raw.objects.insert(id, value);
            }
        }

        LoadFromFile { raw }
    }
}

impl Load for LoadFromFile {
    fn get_headers(&self, header: &str) -> Option<&Value> {
        self.raw.headers.get(header)
    }

    fn get_components(&self, component: &str) -> Vec<(u32, &Value)> {
        self.raw
            .objects
            .iter()
            .flat_map(|(id, value)| match value.get(component) {
                Some(value) => Some((*id, value)),
                None => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::save::{Load, LoadFromFile, Save, SaveToFile};
    use serde_json::json;
    use std::{fs, io};

    #[test]
    pub fn save_and_load_test() {
        let file = "/tmp/test.save";
        let _ = fs::remove_file(file);

        {
            let mut save = SaveToFile::new(file);
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
        }

        {
            let load = LoadFromFile::new(file);
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
