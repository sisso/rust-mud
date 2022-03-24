use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemConf {
    pub code: String,
    pub kind: String,
    pub attack_mod: f32,
    pub defense_mod: f32,
    pub name: String,
    pub rarity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialConf {
    pub code: String,
    pub name: String,
    pub rarity: f32,
    pub bonus: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemsConf {
    pub items: Vec<ItemConf>,
    pub attacks: commons::prob::RDistrib,
    pub defenses: commons::prob::RDistrib,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conf {
    pub items: ItemsConf,
    pub materials: Vec<MaterialConf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub item_code: String,
    pub material_code: String,
    pub attack: i32,
    pub defense: i32,
    pub armor: i32,
    pub hp: i32,
    pub damage: i32,
    pub kind: String,
    pub rarity: f32,
}

pub fn generate_item(conf: &Conf, seed: u64) -> Item {
    let mut rng: rand::prelude::StdRng = rand::prelude::SeedableRng::seed_from_u64(seed);
    let item =
        commons::prob::select_weighted_by(&mut rng, &conf.items.items, |i| i.rarity).unwrap();
    let material =
        commons::prob::select_weighted_by(&mut rng, &conf.materials, |material| material.rarity)
            .unwrap();

    // apply material
    let attack = conf.items.attacks.next(&mut rng) * material.bonus * item.attack_mod;
    let defense = conf.items.defenses.next(&mut rng) * material.bonus * item.defense_mod;

    let name = format!("{} {}", material.name, item.name);

    let rarity = item.rarity * material.rarity * (1.0 / (attack + defense));

    Item {
        name: name,
        item_code: item.code.clone(),
        material_code: material.code.clone(),
        attack: attack as i32,
        defense: defense as i32,
        armor: 0,
        hp: 0,
        damage: 0,
        kind: item.kind.clone(),
        rarity: rarity,
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
                        name: "Sword".to_string(),
                        rarity: 1.0,
                    },
                    ItemConf {
                        code: "sword_long".to_string(),
                        kind: "sword".to_string(),
                        attack_mod: 1.2,
                        defense_mod: 0.0,
                        name: "Long Sword".to_string(),
                        rarity: 1.0,
                    },
                    ItemConf {
                        code: "sword_short".to_string(),
                        kind: "sword".to_string(),
                        attack_mod: 0.8,
                        defense_mod: 0.2,
                        name: "Short Sword".to_string(),
                        rarity: 1.0,
                    },
                    ItemConf {
                        code: "dagger".to_string(),
                        kind: "dagger".to_string(),
                        attack_mod: 0.5,
                        defense_mod: 0.2,
                        name: "Dagger".to_string(),
                        rarity: 1.0,
                    },
                    ItemConf {
                        code: "mail".to_string(),
                        kind: "mail".to_string(),
                        attack_mod: 0.0,
                        defense_mod: 1.0,
                        name: "Mail".to_string(),
                        rarity: 1.0,
                    },
                ],
                attacks: commons::prob::RDistrib::MinMax(0.0, 10.0),
                defenses: commons::prob::RDistrib::MinMax(0.0, 10.0),
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
                    bonus: 1.1,
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

    #[test]
    fn test1() {
        // sample
        let conf = setup();

        let mut list: Vec<Item> = (0..100).map(|i| generate_item(&conf, i)).collect();

        list.sort_by_key(|i| (i.rarity * 10000.0) as i32);

        for (i, e) in list.into_iter().enumerate() {
            println!("{}: {:?}", i, e);
        }
    }
}
