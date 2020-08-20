use crate::controller::{ConnectionView, ConnectionViewAction};
use crate::game::container::Container;
use crate::game::loader;
use crate::game::loader::dto::ObjData;
use crate::utils::strinput::StrInput;
use commons::jsons::JsonValueExtra;
use commons::tree::Tree;
use commons::ObjId;
use logs::*;
use serde_json::Value;
use std::collections::HashMap;

pub fn handle(
    container: &mut Container,
    outputs: &mut Vec<String>,
    input: &str,
) -> crate::errors::Result<ConnectionViewAction> {
    let input = StrInput(input);

    // do action and append outputs
    match input.get_command() {
        "help" => {
            outputs.push("Available commands:".into());
            outputs.push("list get update exit".into());
            Ok(ConnectionViewAction::None)
        }

        "list" => {
            handle_list(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

        "get" => {
            handle_get(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

        "update" => Ok(ConnectionViewAction::None),

        "exit" => {
            // TODO: pop current view?
            Ok(ConnectionViewAction::SwitchView(ConnectionView::Game))
        }
        _ => {
            outputs.push(format!("unknown command [{}]", input.as_str()));
            Ok(ConnectionViewAction::None)
        }
    }
}

fn handle_get(
    container: &mut Container,
    outputs: &mut Vec<String>,
    input: StrInput,
) -> crate::errors::Result<()> {
    let id = match input.parse_arguments().get(0).map(|s| s.parse()) {
        Some(Ok(id)) => ObjId(id),
        _ => {
            outputs.push(format!("invalid number argument"));
            return Ok(());
        }
    };

    let data = container.loader.get_prefab(id.into()).cloned().or_else(|| {
        match loader::Loader::snapshot_obj(container, id) {
            Ok(data) => Some(data),
            Err(e) => {
                warn!("admin fail to to get {:?}: {:?}", id, e);
                None
            }
        }
    });

    match data {
        Some(data) => {
            let mut value: Value = serde_json::to_value(data)?;
            value.strip_nulls();

            let serialized = serde_json::to_string_pretty(&value)?;
            outputs.push("".into());
            outputs.push(serialized);
            Ok(())
        }

        None => {
            outputs.push(format!("fail to get {}", id.as_u32()));
            Ok(())
        }
    }
}

fn handle_list(
    container: &mut Container,
    outputs: &mut Vec<String>,
    input: StrInput,
) -> crate::errors::Result<()> {
    let data = loader::Loader::create_snapshot(container)?;

    let mut data_by_id = HashMap::new();
    let mut roots = vec![];
    let mut roots_prefabs = vec![];
    let mut tree = Tree::<u32>::new();

    for (_, e) in &data.objects {
        match e.parent {
            Some(parent_id) => {
                tree.insert(e.id.as_u32(), parent_id.as_u32());
            }
            None => roots.push(e.id.as_u32()),
        }

        data_by_id.insert(e.id.as_u32(), e);
    }

    for (_, e) in &data.prefabs {
        match e.parent {
            Some(parent_id) => {
                tree.insert(e.id.as_u32(), parent_id.as_u32());
            }
            None => roots_prefabs.push(e.id.as_u32()),
        }

        data_by_id.insert(e.id.as_u32(), e);
    }

    outputs.push("".into());
    outputs.push("Prefabs:".into());

    roots_prefabs.sort();
    for key in roots_prefabs {
        print_deep(outputs, 0, key, &data_by_id, &tree);
    }

    outputs.push("".into());
    outputs.push("Objects:".into());

    roots.sort();
    for key in roots {
        print_deep(outputs, 0, key, &data_by_id, &tree);
    }

    Ok(())
}

fn print_one(deep: u32, data: &ObjData) -> String {
    let prefix = String::from_utf8(vec![b' '; (deep * 2) as usize]).unwrap();
    let mut children_str = "".to_string();
    for children in &data.children {
        let mut ids = children.iter().map(|i| i.as_u32()).collect::<Vec<_>>();
        ids.sort();
        children_str = format!(" - {:?}", ids);
    }
    format!(
        "{}{}) {}{}",
        prefix,
        data.id.as_u32(),
        data.label,
        children_str
    )
}

fn print_deep(
    outputs: &mut Vec<String>,
    deep: u32,
    key: u32,
    data_by_id: &HashMap<u32, &ObjData>,
    tree: &Tree<u32>,
) {
    outputs.push(print_one(deep, data_by_id.get(&key).unwrap()));

    let mut children = tree.children(key).collect::<Vec<_>>();
    children.sort();

    for child in children {
        print_deep(outputs, deep + 1, child, data_by_id, tree);
    }
}

pub fn handle_welcome() -> String {
    "[Admin]".to_string()
}
