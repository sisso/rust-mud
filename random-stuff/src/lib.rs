use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemConf {
    pub code: String,
    pub kind: String,
    pub attack_mod: f32,
    pub defense_mod: f32,
    pub armor_mod: f32,
    pub hp_mod: f32,
    pub damage_min_mod: f32,
    pub damage_max_mod: f32,
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
    pub armor: commons::prob::RDistrib,
    pub hp: commons::prob::RDistrib,
    pub damage_min: commons::prob::RDistrib,
    pub damage_max: commons::prob::RDistrib,
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
    pub damage_min: i32,
    pub damage_max: i32,
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

    fn round(v: f32) -> i32 {
        v.round() as i32
    }

    // apply material
    let attack = round(conf.items.attacks.next(&mut rng) * material.bonus * item.attack_mod);
    let defense = round(conf.items.defenses.next(&mut rng) * material.bonus * item.defense_mod);
    let armor = round(conf.items.armor.next(&mut rng) * material.bonus * item.armor_mod);
    let hp = round(conf.items.hp.next(&mut rng) * material.bonus * item.hp_mod);
    let damage_min =
        round(conf.items.damage_min.next(&mut rng) * material.bonus * item.damage_min_mod);
    let damage_max = round(
        conf.items.damage_max.next(&mut rng) * material.bonus * item.damage_max_mod
            + damage_min as f32,
    );

    let name = format!("{} {}", material.name, item.name);

    let mods_total = attack + defense + hp + armor + damage_min + damage_max;
    if mods_total <= 1 {
        return generate_item(conf, rng.next_u64());
    }

    let rarity = item.rarity * material.rarity * (1.0 / mods_total as f32);

    Item {
        name: name,
        item_code: item.code.clone(),
        material_code: material.code.clone(),
        attack: attack as i32,
        defense: defense as i32,
        armor: armor as i32,
        hp: hp as i32,
        damage_min: damage_min as i32,
        damage_max: damage_max as i32,
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
                items: vec![ItemConf {
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
                }],
                attacks: commons::prob::RDistrib::MinMax(0.0, 10.0),
                defenses: commons::prob::RDistrib::MinMax(0.0, 10.0),
                armor: commons::prob::RDistrib::MinMax(0.0, 10.0),
                hp: commons::prob::RDistrib::MinMax(0.0, 10.0),
                damage_min: commons::prob::RDistrib::MinMax(0.0, 5.0),
                damage_max: commons::prob::RDistrib::MinMax(0.0, 10.0),
            },
            materials: vec![MaterialConf {
                code: "iron".to_string(),
                name: "Iron".to_string(),
                rarity: 1.0,
                bonus: 1.0,
            }],
        }
    }

    #[test]
    fn test1() {
        // sample
        let conf = setup();

        let list: Vec<Item> = (0..100).map(|i| generate_item(&conf, i)).collect();
        assert_eq!(100, list.len());
    }
}
