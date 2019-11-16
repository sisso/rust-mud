use super::*;

pub struct MenuView {

}

impl MenuView {
    pub fn new() -> Self {
        MenuView {}
    }
}

impl View for MenuView {
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData) {
        view_manager.output(data.connection_id, comm::menu_welcome());
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        match input {
            "1" => {
                data.current = ViewKind::Game;
                ViewAction::ChangeView
            },
            "2" => {
                view_manager.disconnect(data.connection_id);
                ViewAction::None
            },
            _other => {
                view_manager.output(data.connection_id, comm::menu_invalid(input));
                view_manager.output(data.connection_id, comm::menu_welcome());
                ViewAction::None
            },
        }
    }

    fn handle_events(&mut self, _view_manager: &mut dyn ViewController, _data: &mut ViewData, _events: &Vec<Event>) -> ViewAction {
        ViewAction::None
    }
}
