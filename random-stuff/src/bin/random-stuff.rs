use random_stuff::*;

fn setup() -> Conf {
    // env_logger::builder()
    //     .filter(None, log::LevelFilter::Trace)
    //     .is_test(true)
    //     .default_format()
    //     .init();

    Conf {
        items: ItemsConf {
            items: vec![
                ItemConf {
                    code: "sword".to_string(),
                    kind: "sword".to_string(),
                    attack_mod: 1.0,
                    defense_mod: 0.1,
                    armor_mod: 0.0,
                    hp_mod: 0.0,
                    damage_min_mod: 1.0,
                    damage_max_mod: 1.0,
                    name: "Sword".to_string(),
                    rarity: 1.0,
                },
                ItemConf {
                    code: "sword_long".to_string(),
                    kind: "sword".to_string(),
                    attack_mod: 1.4,
                    defense_mod: 0.0,
                    armor_mod: 0.0,
                    hp_mod: 0.0,
                    damage_min_mod: 1.1,
                    damage_max_mod: 1.2,
                    name: "Long Sword".to_string(),
                    rarity: 0.7,
                },
                ItemConf {
                    code: "sword_short".to_string(),
                    kind: "sword".to_string(),
                    attack_mod: 0.8,
                    defense_mod: 0.2,
                    armor_mod: 0.0,
                    hp_mod: 0.0,
                    damage_min_mod: 1.0,
                    damage_max_mod: 0.8,
                    name: "Short Sword".to_string(),
                    rarity: 1.0,
                },
                ItemConf {
                    code: "dagger".to_string(),
                    kind: "dagger".to_string(),
                    attack_mod: 0.5,
                    defense_mod: 0.2,
                    armor_mod: 0.0,
                    hp_mod: 0.0,
                    damage_min_mod: 0.5,
                    damage_max_mod: 0.5,
                    name: "Dagger".to_string(),
                    rarity: 1.0,
                },
                ItemConf {
                    code: "mail".to_string(),
                    kind: "mail".to_string(),
                    attack_mod: 0.0,
                    defense_mod: 1.0,
                    armor_mod: 1.0,
                    hp_mod: 0.1,
                    damage_min_mod: 0.0,
                    damage_max_mod: 0.0,
                    name: "Mail".to_string(),
                    rarity: 0.8,
                },
                ItemConf {
                    code: "vest".to_string(),
                    kind: "vest".to_string(),
                    attack_mod: 0.0,
                    defense_mod: 0.25,
                    armor_mod: 0.25,
                    hp_mod: 1.0,
                    damage_min_mod: 0.0,
                    damage_max_mod: 0.0,
                    name: "Mail".to_string(),
                    rarity: 1.0,
                },
            ],
            attacks: commons::prob::RDistrib::MinMax(0.0, 10.0),
            defenses: commons::prob::RDistrib::MinMax(0.0, 10.0),
            armor: commons::prob::RDistrib::MinMax(0.0, 10.0),
            hp: commons::prob::RDistrib::MinMax(0.0, 10.0),
            damage_min: commons::prob::RDistrib::MinMax(0.0, 5.0),
            damage_max: commons::prob::RDistrib::MinMax(0.0, 10.0),
        },
        materials: vec![
            MaterialConf {
                code: "iron".to_string(),
                name: "Iron".to_string(),
                rarity: 1.0,
                bonus: 1.0,
            },
            MaterialConf {
                code: "steel".to_string(),
                name: "Steel".to_string(),
                rarity: 0.5,
                bonus: 1.2,
            },
            MaterialConf {
                code: "copper".to_string(),
                name: "Copper".to_string(),
                rarity: 1.0,
                bonus: 0.7,
            },
        ],
    }
}
fn main() {
    let conf = setup();
    let mut list: Vec<Item> = (0..100).map(|i| generate_item(&conf, i)).collect();
    list.sort_by_key(|i| (i.rarity * 10000.0) as i32);

    println!("index,kind,name,attack,defense,damage_min,damage_max,armor,hp,rarity,item_code,material_code");
    for (i, e) in list.into_iter().enumerate() {
        println!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            i,
            e.kind,
            e.name,
            e.attack,
            e.defense,
            e.damage_min,
            e.damage_max,
            e.armor,
            e.hp,
            e.rarity,
            e.item_code,
            e.material_code,
        );
    }
}
