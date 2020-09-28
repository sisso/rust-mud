use crate::controller::{ConnectionView, ConnectionViewAction};
use crate::errors::*;
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

pub fn handle_welcome() -> String {
    "[Admin]\n".to_string()
}

pub fn handle(
    container: &mut Container,
    outputs: &mut Vec<String>,
    input: &str,
) -> Result<ConnectionViewAction> {
    let input = StrInput(input);

    // do action and append outputs
    match input.get_command() {
        "help" => {
            outputs.push("Available commands:".into());
            outputs.push("list get update remove exit".into());
            outputs.push("".into());
            Ok(ConnectionViewAction::None)
        }

        "list" | "ls" => {
            handle_list(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

        "get" | "g" => {
            handle_get(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

        "add" => {
            handle_add(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

        "remove" => {
            handle_remove(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

        "update" => Ok(ConnectionViewAction::None),

        "verify" => {
            handle_verify(container, outputs, input)?;
            Ok(ConnectionViewAction::None)
        }

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

fn handle_get(container: &mut Container, outputs: &mut Vec<String>, input: StrInput) -> Result<()> {
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
) -> Result<()> {
    // build data structure
    let data = loader::Loader::create_snapshot(container)?;

    let mut data_by_id = HashMap::new();
    let mut roots = vec![];
    let mut roots_prefabs = vec![];
    let mut tree = Tree::<u32>::new();

    for (_, e) in &data.objects {
        match e.parent {
            Some(parent_id) => {
                tree.insert(e.get_id().as_u32(), parent_id.as_u32());
            }
            None => roots.push(e.get_id().as_u32()),
        }

        data_by_id.insert(e.get_id().as_u32(), e);
    }

    for (_, e) in &data.prefabs {
        match e.parent {
            Some(parent_id) => {
                tree.insert(e.get_id().as_u32(), parent_id.as_u32());
            }
            None => roots_prefabs.push(e.get_id().as_u32()),
        }

        data_by_id.insert(e.get_id().as_u32(), e);
    }

    // search valid objects
    let mut args: Vec<_> = input
        .parse_arguments()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let mut show_prefabs = true;
    let mut show_objects = true;

    match args.get(0).map(|s| s.as_str()) {
        Some("p") => {
            show_objects = false;
            args.remove(0);
        }

        Some("o") => {
            show_prefabs = false;
            args.remove(0);
        }

        _ => {}
    }

    let filters = VecStringFilter::new(args);
    info!("creating filter {:?}", filters);

    // display
    if show_prefabs {
        outputs.push("".into());
        outputs.push("Prefabs:".into());

        roots_prefabs.sort();
        for key in roots_prefabs {
            print_deep(&filters, outputs, 0, key, &data_by_id, &tree);
        }
    }

    if show_objects {
        outputs.push("".into());
        outputs.push("Objects:".into());

        roots.sort();
        for key in roots {
            print_deep(&filters, outputs, 0, key, &data_by_id, &tree);
        }
    }

    Ok(())
}

fn handle_verify(
    container: &mut Container,
    outputs: &mut Vec<String>,
    input: StrInput,
) -> Result<()> {
    let json = input.plain_arguments();

    match serde_json::from_str::<ObjData>(json) {
        Ok(data) => {
            let s = to_string_pretty(&data)?;
            outputs.push(s);
            Ok(())
        }

        Err(e) => {
            outputs.push(format!("fail to parse: {:?}", e));
            Ok(())
        }
    }
}

fn to_string_pretty(data: &ObjData) -> Result<String> {
    let mut data = serde_json::to_value(data)?;
    data.strip_nulls();

    let string = serde_json::to_string_pretty(&data)?;
    Ok(string)
}

fn handle_add(container: &mut Container, outputs: &mut Vec<String>, input: StrInput) -> Result<()> {
    let plains = input.plain_arguments();
    let is_prefab = plains.starts_with("p ");
    let is_obj = plains.starts_with("o ");
    if !is_prefab && !is_obj {
        outputs.push("you need to specific p or o as first argument".to_string());
        return Ok(());
    }

    let json = &plains[2..];

    match serde_json::from_str::<ObjData>(json) {
        Ok(mut data) => {
            let obj_id = container.objects.create();

            if is_obj {
                loader::Loader::apply_objdata(container, obj_id, &data);
            } else {
                data.id = Some(obj_id.into());
                container.loader.add_prefab(data);
            }

            outputs.push(format!("created id: {}", obj_id.as_u32()));

            Ok(())
        }

        Err(e) => {
            outputs.push(format!("fail to parse: {:?}", e));
            Ok(())
        }
    }
}

trait Filter {
    fn is_visible(&self, data: &ObjData) -> bool;
}

#[derive(Debug)]
struct VecStringFilter {
    labels: Vec<String>,
    is_mob: Option<bool>,
    is_item: Option<bool>,
    is_room: Option<bool>,
}

impl VecStringFilter {
    pub fn new(mut args: Vec<String>) -> Self {
        fn drain(v: &mut Vec<String>, s: &str) -> Option<bool> {
            match v.iter().position(|i| i.as_str() == s) {
                Some(pos) => {
                    v.remove(pos);
                    Some(true)
                }

                _ => None,
            }
        }

        let is_mob = drain(&mut args, "mob");
        let is_room = drain(&mut args, "room");
        let is_item = drain(&mut args, "item");

        VecStringFilter {
            labels: args,
            is_mob,
            is_item,
            is_room,
        }
    }

    fn is_valid_label(&self, data: &ObjData) -> bool {
        for s in self.labels.iter() {
            let is_label = data.label.as_ref().map(|l| l.contains(s)).unwrap_or(false);
            let is_code = data
                .code
                .as_ref()
                .map(|v| {
                    for i in v.iter() {
                        if i.contains(s) {
                            return true;
                        }
                    }

                    return false;
                })
                .unwrap_or(false);

            let is_id = format!("{}", data.get_id().as_u32()).contains(s);

            let is_valid = is_label || is_code || is_id;

            if !is_valid {
                return false;
            }
        }

        return true;
    }

    fn is_valid_tags(&self, data: &ObjData) -> bool {
        if self.is_room.unwrap_or(false) && !data.room.is_some() {
            return false;
        }

        if self.is_mob.unwrap_or(false) && !data.mob.is_some() {
            return false;
        }

        if self.is_item.unwrap_or(false) && !data.item.is_some() {
            return false;
        }

        return true;
    }
}

impl Filter for VecStringFilter {
    fn is_visible(&self, data: &ObjData) -> bool {
        self.is_valid_label(data) && self.is_valid_tags(data)
    }
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
        data.get_id().as_u32(),
        data.label.as_ref().unwrap_or(&"undefined".to_string()),
        children_str
    )
}

fn print_deep(
    filters: &dyn Filter,
    outputs: &mut Vec<String>,
    deep: u32,
    key: u32,
    data_by_id: &HashMap<u32, &ObjData>,
    tree: &Tree<u32>,
) {
    let data = data_by_id.get(&key).unwrap();
    if filters.is_visible(data) {
        outputs.push(print_one(deep, data));
    }

    let mut children = tree.children(key).collect::<Vec<_>>();
    children.sort();

    for child in children {
        print_deep(filters, outputs, deep + 1, child, data_by_id, tree);
    }
}

fn handle_remove(
    container: &mut Container,
    outputs: &mut Vec<String>,
    input: StrInput,
) -> Result<()> {
    let id = match input.parse_arguments().get(0).map(|s| s.parse()) {
        Some(Ok(id)) => ObjId(id),
        _ => {
            outputs.push(format!("invalid number argument"));
            return Ok(());
        }
    };

    container.remove(id);
    outputs.push(format!("object {} removed", id.as_u32()));
    Ok(())
}
