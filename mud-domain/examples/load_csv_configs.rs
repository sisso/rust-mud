use commons::csv::{
    csv_strings_to_tables, parse_csv, tables_to_json, tables_to_jsonp, FieldKind, Table,
};
use mud_domain::game::container::Container;
use mud_domain::game::loader::{
    Data, ItemArmorData, ItemData, ItemWeaponData, Loader, ObjData, PriceData, StaticId,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

fn main() {
    let json = {
        // let file = std::env::args().collect::<Vec<_>>();
        let string = std::fs::read_to_string(Path::new("data/fantasy_2/config-01.csv"))
            .expect("fail to open file");
        let csv = parse_csv(string.as_str());
        let tables = csv_strings_to_tables(&csv).expect("fail to parse tables");

        let mut parsers = HashMap::new();
        parsers.insert("item_weapon_attack", FieldKind::I32);
        parsers.insert("item_weapon_defense", FieldKind::I32);
        parsers.insert("item_weapon_damage_max", FieldKind::U32);
        parsers.insert("item_weapon_damage_min", FieldKind::U32);
        parsers.insert("item_weapon_calmdown", FieldKind::F32);
        parsers.insert("item_armor_defense", FieldKind::I32);
        parsers.insert("item_armor_rd", FieldKind::I32);
        parsers.insert("price_buy", FieldKind::U32);

        tables_to_jsonp(&tables, &parsers).unwrap()
    };

    let mut container = Container::new();
    Loader::load_json_flat(&mut container, json).unwrap();

    println!("done");
}
