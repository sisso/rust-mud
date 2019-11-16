use super::*;

pub struct CharacterCreationView {

}

impl CharacterCreationView {
    pub fn new() -> Self {
        CharacterCreationView {}
    }
}

impl View for CharacterCreationView {
    fn init(&mut self, _view_manager: &mut dyn ViewController, _data: &mut ViewData) {
    }

    fn handle(&mut self, _view_manager: &mut dyn ViewController, _input: &str, _data: &mut ViewData) -> ViewAction {
        // TODO: query game classes
        // TODO: query game races
        // TODO: create character for player

        ViewAction::None
    }

    fn handle_events(&mut self, _view_manager: &mut dyn ViewController, _data: &mut ViewData, _events: &Vec<Event>) -> ViewAction {
        ViewAction::None
    }
}

