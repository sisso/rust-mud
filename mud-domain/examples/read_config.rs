use std::env;
use mud_domain::game::loader::Loader;
use std::path::Path;
use std::collections::HashMap;

// TODO: print a tree
fn main() {
    let path = env::args().nth(1).unwrap_or("data/space".to_string());
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

    println!("List");
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
        println!("{}) {} {}{}", key, label, is_prefab_str, has_parent);
    }

    println!();
    println!("next: {}", max + 1);
    println!();
    println!("Legend:");
    println!("* is prefab");
    println!("$ is root object, has no parent");
}
