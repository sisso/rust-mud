use core::utils::UserId;
use mud_engine::Engine;

use crate::command_line_controller::*;


pub fn init_login(engine: &mut Engine, outputs: &mut dyn Outputs, view: &mut ViewData) {
    outputs.add(view.connection_id, "Welcome to the mud.\
     \
       login: ".to_string());
}

pub fn handle_input(engine: &mut Engine, outputs: &mut dyn Outputs, view: &mut ViewData, input: String) {
    match view.current {
        ViewKind::Login => handle_login(engine, outputs, view, input),
        _ => unimplemented!()
    }
}

fn handle_login(engine: &mut Engine, outputs: &mut dyn Outputs, view: &mut ViewData, input: String) {
//    match (view.login_data.login, view.login_data.password) {
//        (None, _) => {
//            outputs.add(view.connection_id, "login: ".to_string());
//
//        },
//        (_, None) => {
//            outputs.add(view.connection_id, "login: ".to_string());
//        },
//        (Some(login), Some(pass)) => {
//            if engine.login(view.connection_id, login.as_str(), pass.as_str()) {
//                // TODO: remove pass from collection
//            }
//        },
//    }
}
