use super::*;

pub struct GameView {

}

impl GameView {
    pub fn new() -> Self {
        GameView {}
    }
}

impl View for GameView {
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData) {
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        view_manager.output(data.connection_id, comm::unknown_input(input));
        ViewAction::None
    }
}
