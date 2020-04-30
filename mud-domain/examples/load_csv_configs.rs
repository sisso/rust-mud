use commons::csv::{csv_strings_to_tables, parse_csv, tables_to_json, Table};
use mud_domain::game::container::Container;
use mud_domain::game::loader::{
    Data, ItemArmorData, ItemData, ItemWeaponData, Loader, ObjData, PriceData, StaticId,
};
use std::path::Path;
use std::str::FromStr;

fn main() {
    let tables = {
        // let file = std::env::args().collect::<Vec<_>>();
        let string = std::fs::read_to_string(Path::new("data/fantasy/config-01.csv"))
            .expect("fail to open file");
        let csv = parse_csv(string.as_str());
        csv_strings_to_tables(&csv).expect("fail to parse tables")
    };

    let container = load_tables(&tables).unwrap();
    // println!("{:?}", container);
    println!("done");
}

fn load_tables(tables: &Vec<Table>) -> Result<Container, String> {
    let mut data = Data::new();
    let mut container = Container::new();
    // let mut static_ids = vec![];
    //
    // for table in tables {
    //     match table.name.as_str() {
    //         "weapons" => {
    //             for i in 0..table.rows.len() {
    //                 let static_id = table.get(i, "static_id");
    //                 static_ids.push(static_id);
    //                 let static_id = StaticId(static_ids.len() as u32);
    //
    //                 let label = table.get(i, "label");
    //                 let attack = as_i32(table.get(i, "attack"))?;
    //                 let defense = as_i32(table.get(i, "defense"))?;
    //                 let damage = as_i32(table.get(i, "damage"))?;
    //                 let recharge = as_f32(table.get(i, "recharge"))?;
    //                 let price = as_u32(table.get(i, "price"))?;
    //
    //                 let mut obj = ObjData::new(static_id);
    //                 obj.label = label.to_string();
    //                 obj.item = Some(ItemData {
    //                     flags: None,
    //                     amount: None,
    //                     weapon: Some(ItemWeaponData {
    //                         min: damage as u32,
    //                         max: damage as u32,
    //                         calm_down: recharge,
    //                         attack: attack,
    //                         defense: defense,
    //                     }),
    //                     armor: None,
    //                 });
    //                 obj.price = Some(PriceData {
    //                     buy: price as u32,
    //                     sell: (price / 2) as u32,
    //                 });
    //
    //                 data.prefabs.insert(static_id, obj);
    //             }
    //         }
    //         "mobs" => {}
    //         "loots" => {}
    //         "armors" => {
    //             let static_id = table.get(i, "static_id");
    //             static_ids.push(static_id);
    //             let static_id = StaticId(static_ids.len() as u32);
    //
    //             let label = table.get(i, "label");
    //             let defense = as_i32(table.get(i, "defense"))?;
    //             let rd = as_u32(table.get(i, "rd"))?;
    //             let price = as_u32(table.get(i, "price"))?;
    //
    //             let mut obj = ObjData::new(static_id);
    //             obj.label = label.to_string();
    //             obj.item = Some(ItemData {
    //                 flags: None,
    //                 amount: None,
    //                 weapon: None,
    //                 armor: Some(ItemArmorData {
    //                     defense: defense,
    //                     rd: rd,
    //                 }),
    //             });
    //             obj.price = Some(PriceData {
    //                 buy: price as u32,
    //                 sell: (price / 2) as u32,
    //             });
    //
    //             data.prefabs.insert(static_id, obj);
    //         }
    //         "shields" => {}
    //         other => {
    //             return Err(format!("Table name {:?} is unknown", other));
    //         }
    //     }
    // }
    //
    // Loader::load_data(&mut container, data).unwrap();
    Ok(container)
}

pub fn as_i32(s: &str) -> Result<i32, String> {
    i32::from_str(s).map_err(|err| format!("fail to parse {:?} {:?}", s, err))
}

pub fn as_u32(s: &str) -> Result<u32, String> {
    u32::from_str(s).map_err(|err| format!("fail to parse {:?} {:?}", s, err))
}

pub fn as_f32(s: &str) -> Result<f32, String> {
    f32::from_str(s).map_err(|err| format!("fail to parse {:?} {:?}", s, err))
}
