use serde_json::{json, Value};
use serde::{Serialize, Deserialize};

use std::fs::File;

use std::io::{Write, BufReader};
use std::collections::HashMap;



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
    fn flush(&mut self);
}

pub trait Load {
    fn get_headers(&self, header: &str) -> Option<&serde_json::Value>;
    fn get_components(&self, component: &str) -> Vec<(u32, &serde_json::Value)>;
}

pub struct SaveToFile {
    file_path: String,
    raw: RawData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        self.raw.headers.insert(header_name.to_string(), header_value);
    }

    fn add(&mut self, id: u32, component: &str, value: serde_json::Value) {
        let obj_map = self.raw.objects.entry(id).or_default();
        obj_map.insert(component.to_string(), value);
    }

    fn flush(&mut self) {
        let mut file = File::create(self.file_path.to_string()).unwrap();
        let json_data = json!(self.raw).to_string();
        let _ = file.write_all(json_data.as_str().as_bytes()).unwrap();
        let _ = file.flush().unwrap();
    }
}

pub struct LoadFromFile {
    raw: RawData,
}

impl LoadFromFile {
    pub fn new(file_path: &str) -> Self {
       let file = File::open(file_path).expect(&format!("failed to open file {:?}", file_path));
        let raw: RawData = serde_json::from_reader(BufReader::new(file)).unwrap();
        LoadFromFile {
            raw,
        }
    }
}

impl Load for LoadFromFile {
    fn get_headers(&self, header: &str) -> Option<&Value> {
        self.raw.headers.get(header)
    }

    fn get_components(&self, component: &str) -> Vec<(u32, &Value)> {
        self.raw.objects.iter().flat_map(|(id, value)| {
            match value.get(component) {
                Some(value) => Some((*id, value)),
                None => None,
            }
        }).collect()
    }
}

#[cfg(test)]
mod test {
    use std::{io, fs};
    use serde_json::json;
    use crate::save::{SaveToFile, Save, LoadFromFile, Load};

    #[test]
    pub fn save_and_load_test() -> io::Result<()> {
        let file = "/tmp/test.save";
        let _ = fs::remove_file(file);

        let mut save = SaveToFile::new(file);
        save.add_header("start_room", json!(22));
        save.add(0, "label", json!({
            "label": "Room 1",
            "desc": "No description"
        }));
        save.add(0, "room", json!({
            "exits": [
                { "dir": "n", "id": 1 }
            ]
        }));
        save.add(1, "label", json!({
            "label": "Room 2",
            "desc": "No description"
        }));
        save.flush();


        let load = LoadFromFile::new(file);
        let labels = load.get_components("label");

        assert_eq!(22 as i64, load.get_headers("start_room").unwrap().as_i64().unwrap());

        assert_eq!(2, labels.len());
        match labels.first() {
            Some((id, value)) if *id == 0 => {
                let value = value["label"].as_str().unwrap();
                assert_eq!("Room 1", value);
            },
            Some((id, value)) if *id == 1 => {
                let value = value["label"].as_str().unwrap();
                assert_eq!("Room 2", value);
            },
            _ => panic!()
        }

        Ok(())
    }
}
