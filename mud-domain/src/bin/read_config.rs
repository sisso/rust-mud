use mud_domain::game::loader::{dto::ObjData, Loader};

use commons::tree::Tree;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::env;

use clap::{self, Clap};
use commons::{asciicolors, jsons::JsonValueExtra};
use std::path::Path;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
struct Opts {
    #[clap(about = "Directory to read all files or a json file")]
    path: String,
    #[clap(short, long, about = "obj id to describe")]
    obj_id: Option<u32>,
    #[clap(short, long, about = "prefab id to describe")]
    prefab_id: Option<u32>,
}

fn main() {
    let opts: Opts = Opts::parse();
    let path = opts.path;
    let dump_obj_id = opts.obj_id;
    let dump_prefab_id = opts.prefab_id;
    let is_dump = dump_obj_id.is_some() || dump_prefab_id.is_some();

    let path = Path::new(path.as_str());

    let mut data = if path.is_dir() {
        Loader::read_folders(path).expect("fail to load directory")
    } else if path.exists() {
        Loader::read_files(vec![path]).expect("fail to load file")
    } else {
        eprintln!("file or directory [{:?}] not found", path);
        std::process::exit(2);
    };

    let mut dataobj_by_id = HashMap::new();
    let mut dataprefab_by_id = HashMap::new();
    let mut roots = vec![];
    let mut roots_prefabs = vec![];
    let mut tree_obj = Tree::<u32>::new();
    let mut tree_prefab = Tree::<u32>::new();
    let mut max_id = 0;
    let mut max_prefab_id = 0;
    let mut errors = vec![];
    let mut broken_set: HashSet<u32> = HashSet::new();

    let validation_result =
        Loader::validate_and_normalize(&mut data).expect("fail to validate data");

    for (id_a, id_b) in &validation_result.mismatch_ids {
        broken_set.insert(id_a.as_u32());
        broken_set.insert(id_b.as_u32());
        errors.push(format!("mismatch ids {:?} and {:?}", id_a, id_b));
    }

    for id in &validation_result.duplicate_ids {
        broken_set.insert(id.as_u32());
        errors.push(format!("duplicate id {:?}", id));
    }

    for (_static_id, e) in &data.objects {
        match e.parent {
            Some(parent_id) => {
                tree_obj.insert(e.get_id().as_u32(), parent_id.as_u32());
            }
            None => roots.push(e.get_id().as_u32()),
        }

        max_id = max_id.max(e.get_id().as_u32());
        dataobj_by_id.insert(e.get_id().as_u32(), e);
    }

    for (_, e) in &data.prefabs {
        match e.parent {
            Some(parent_id) => {
                tree_prefab.insert(e.get_id().as_u32(), parent_id.as_u32());
            }
            None => roots_prefabs.push(e.get_id().as_u32()),
        }

        max_prefab_id = max_prefab_id.max(e.get_id().as_u32());

        dataprefab_by_id.insert(e.get_id().as_u32(), e);
    }

    let mut free_prefab_ids = vec![];
    for id in 0..max_prefab_id {
        if !dataprefab_by_id.contains_key(&id) {
            free_prefab_ids.push(id);
        }
    }

    if !is_dump {
        println!();
        println!("Objects:");

        roots.sort();
        for key in roots {
            print_deep(0, key, &dataobj_by_id, &tree_obj, &broken_set);
        }

        println!();
        println!("Prefabs:");

        roots_prefabs.sort();
        for key in roots_prefabs {
            print_deep(0, key, &dataprefab_by_id, &tree_prefab, &broken_set);
        }

        println!();
        println!("Free prefab ids:");
        println!("{:?}", free_prefab_ids);
    }

    if let Some(id) = dump_obj_id {
        let obj = dataobj_by_id.get(&id).unwrap();
        let mut value = serde_json::to_value(&obj).expect("fail to serialize object into value");
        value.strip_nulls();

        let json = serde_json::to_string(&value).expect("Failed to serialize object");
        println!("{}", json);
    } else if let Some(id) = dump_prefab_id {
        let obj = dataprefab_by_id.get(&id).unwrap();
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

fn print_one(deep: u32, data: &ObjData, is_fail: bool) {
    let prefix = String::from_utf8(vec![b' '; (deep * 2) as usize]).unwrap();
    let mut children_str = "".to_string();
    for children in &data.children {
        let mut ids = children.iter().map(|i| i.as_u32()).collect::<Vec<_>>();
        ids.sort();
        children_str = format!(" - {:?}", ids);
    }

    if is_fail {
        print!("{}", asciicolors::fg(asciicolors::COLOR_RED));
    }

    println!(
        "{}{:04} - {}{}",
        prefix,
        data.get_id().as_u32(),
        data.label.as_ref().unwrap_or(&"undefined".to_string()),
        children_str
    );

    if is_fail {
        print!("{}", asciicolors::RESET);
    }
}

fn print_deep(
    deep: u32,
    key: u32,
    data_by_id: &HashMap<u32, &ObjData>,
    tree: &Tree<u32>,
    broken_set: &HashSet<u32>,
) {
    print_one(
        deep,
        data_by_id
            .get(&key)
            .expect(&format!("could not found data for id {}", key)),
        broken_set.contains(&key),
    );

    let mut children = tree.children(key).collect::<Vec<_>>();
    children.sort();

    for child in children {
        print_deep(deep + 1, child, data_by_id, tree, broken_set);
    }
}
