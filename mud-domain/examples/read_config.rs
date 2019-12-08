use std::env;
use mud_domain::game::loader::Loader;
use std::path::Path;
use std::collections::HashMap;

// TODO: print a tree
fn main() {
    let path = env::args().nth(1).expect("argument not provided");
    let data = Loader::read_folder(Path::new(path.as_str())).unwrap();

    let mut m = HashMap::new();

    for (_, data) in &data.objects {
        if m.insert(data.id, (&data.label, false)).is_some() {
            panic!("Duplicated id {:?}", data.id);
        }
    }

    for (_, data) in &data.prefabs {
        if m.insert(data.id, (&data.label, true)).is_some() {
            panic!("Duplicated id {:?}", data.id);
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

        let (label, is_prefab) = m.get(key).unwrap();
        let is_prefab_str = if *is_prefab { "*" } else { "" };
        println!("{}) {} {}", key, label, is_prefab_str);
    }

    println!();
    println!("next: {}", max + 1);
}
