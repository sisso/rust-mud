use mud_domain::game::loader::{dto::ObjData, Loader};

use commons::tree::Tree;
use serde_json;
use std::collections::HashMap;
use std::env;

use commons::jsons::JsonValueExtra;
use std::path::Path;

// TODO: refactory everything, it got ugly,
// TODO: support commands
// TODO: dump ID should be able to pipe to jq
// TODO: show hierarchic as tree

fn usage() {
    println!();
    println!("Usage:");
    println!();
    println!("{} config-folder [id]", env::args().nth(0).unwrap());
    println!();
}

fn main() {
    if env::args().len() < 1 {
        usage();
        std::process::exit(1);
    }

    // TODO: require a proper argument parser
    let path = env::args().nth(1).unwrap_or("data/space".to_string());
    let dump_id: Option<u32> = env::args().nth(2).map(|s| match s.parse() {
        Ok(id) => id,
        _ => {
            eprintln!("Invalid id {:?}", s);
            usage();
            std::process::exit(1);
        }
    });

    let path = Path::new(path.as_str());

    let data = if path.is_dir() {
        Loader::read_folders(path).expect("fail to load directory")
    } else if path.exists() {
        Loader::read_files(vec![path]).expect("fail to load file")
    } else {
        eprintln!("file or directory [{:?}] not found", path);
        std::process::exit(2);
    };

    let mut data_by_id = HashMap::new();
    let mut roots = vec![];
    let mut roots_prefabs = vec![];
    let mut tree = Tree::<u32>::new();
    let mut max_id = 0;
    let mut max_prefab_id = 0;
    let mut errors = vec![];

    for (_, e) in &data.objects {
        match e.parent {
            Some(parent_id) => {
                tree.insert(e.get_id().as_u32(), parent_id.as_u32());
            }
            None => roots.push(e.get_id().as_u32()),
        }

        max_id = max_id.max(e.get_id().as_u32());
        if data_by_id.insert(e.get_id().as_u32(), e).is_some() {
            errors.push(format!("duplicate id {:?}", e.get_id()));
        }
    }

    for (_, e) in &data.prefabs {
        match e.parent {
            Some(parent_id) => {
                tree.insert(e.get_id().as_u32(), parent_id.as_u32());
            }
            None => roots_prefabs.push(e.get_id().as_u32()),
        }

        max_prefab_id = max_prefab_id.max(e.get_id().as_u32());

        if data_by_id.insert(e.get_id().as_u32(), e).is_some() {
            errors.push(format!("duplicate id {:?}", e.get_id()));
        }
    }

    if dump_id.is_none() {
        println!();
        println!("Prefabs:");

        roots_prefabs.sort();
        for key in roots_prefabs {
            print_deep(0, key, &data_by_id, &tree);
        }

        println!();
        println!("Objects:");

        roots.sort();
        for key in roots {
            print_deep(0, key, &data_by_id, &tree);
        }
    }

    if let Some(id) = dump_id {
        let obj = data_by_id.get(&id).unwrap();
        let mut value = serde_json::to_value(&obj).expect("fail to serialize object into value");
        value.strip_nulls();

        let json = serde_json::to_string(&value).expect("Failed to serialize object");
        println!("{}", json);
    } else {
        println!();
        println!("next id: {}", max_id + 1);
        println!("next prefab id: {}", max_prefab_id + 1);
        println!();
        if !errors.is_empty() {
            println!("ERRORS:");
            for e in errors {
                println!("- {}", e);
            }
        }
    }
}

fn print_one(deep: u32, data: &ObjData) {
    let prefix = String::from_utf8(vec![b' '; (deep * 2) as usize]).unwrap();
    let mut children_str = "".to_string();
    for children in &data.children {
        let mut ids = children.iter().map(|i| i.as_u32()).collect::<Vec<_>>();
        ids.sort();
        children_str = format!(" - {:?}", ids);
    }
    println!(
        "{}{:04}) {}{}",
        prefix,
        data.get_id().as_u32(),
        data.label.as_ref().unwrap_or(&"undefined".to_string()),
        children_str
    );
}

fn print_deep(deep: u32, key: u32, data_by_id: &HashMap<u32, &ObjData>, tree: &Tree<u32>) {
    print_one(deep, data_by_id.get(&key).unwrap());

    let mut children = tree.children(key).collect::<Vec<_>>();
    children.sort();

    for child in children {
        print_deep(deep + 1, child, data_by_id, tree);
    }
}
