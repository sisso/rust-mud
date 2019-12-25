use std::env;
use mud_domain::game::loader::Loader;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize};
use serde_json;


// TODO: refactory everything, it got ugly, 
// TODO: support commands
// TODO: dump ID should be able to pipe to jq
// TODO: show hierarchic as tree


fn usage() {
    println!("");
    println!("Usage:");
    println!("");
    println!("{} config-folder [id]", env::args().nth(0).unwrap());
    println!("");
}

fn main() {
    if env::args().len() < 1 {
        usage();
        std::process::exit(1);
    }

    let path = env::args().nth(1).unwrap_or("data/space".to_string());
    let dump_id: Option<u32> = env::args().nth(2)
        .map(|s| { 
            match s.parse() {
                Ok(id) => id,
                _ => { 
                    eprintln!("Invalid id {:?}", s);
                    usage();
                    std::process::exit(1);
                }
            }
        });

    let data = Loader::read_folder(Path::new(path.as_str())).unwrap();

    let mut m = HashMap::new();

    for (_, e) in data.objects.iter() {
        if m.insert(e.id, (&e.label, false, e)).is_some() {
            panic!("Duplicated id {:?}", e.id);
        }
    }

    for (_, e) in data.prefabs.iter() {
        if m.insert(e.id, (&e.label, true, e)).is_some() {
            panic!("Duplicated id {:?}", e.id);
        }
    }

    let mut keys = m.keys().collect::<Vec<&u32>>();
    keys.sort();

    let mut max: u32 = 0;
    for key in keys {
        if *key > max {
            max = *key;
        }

        let (label, is_prefab, obj) = m.get(key).unwrap();
        let is_prefab_str = if *is_prefab { "*" } else { "" };
        let has_parent = if obj.parent.is_some() { "" } else { "$" };
        if dump_id.is_none() {
            println!("{}) {} {}{}", key, label, is_prefab_str, has_parent);
        }
    }

    if let Some(id) = dump_id {
        let (_, _, obj) = m.get(&id).unwrap();
        println!();
        println!("[Dump {:?}]", id);

        let json = serde_json::to_string(&obj).expect("Failed to serialize object");
        println!("{}", json);
    } else {
        println!();
        println!("next: {}", max + 1);
        println!();
        println!("Legend:");
        println!("* is prefab");
        println!("$ is root object, has no parent");
    }
}
