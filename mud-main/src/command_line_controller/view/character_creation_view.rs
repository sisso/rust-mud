use super::*;

pub struct CharacterCreationView {

}

impl CharacterCreationView {
    pub fn new() -> Self {
        CharacterCreationView {}
    }
}

impl View for CharacterCreationView {
    fn init(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData) {
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        // TODO: query game classes
        // TODO: query game races
        // TODO: create character for player

        ViewAction::None
    }
}

