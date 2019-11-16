use mud_engine::Action;
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
        view_manager.output(data.connection_id, "".to_string());
    }

    fn handle(&mut self, view_manager: &mut dyn ViewController, input: &str, data: &mut ViewData) -> ViewAction {
        let player_id = data.player_id.unwrap();
        let action = Action::Generic { input: input.to_string() };
        view_manager.emit(player_id, action);
        ViewAction::None
    }

    fn handle_events(&mut self, view_manager: &mut dyn ViewController, data: &mut ViewData, events: &Vec<Event>) -> ViewAction {
        for event in events.iter() {
            match event {
                Event::Generic { msg } => {
                    view_manager.output(data.connection_id, msg.to_string());
                },
                _ => panic!(),
            }
        }

        ViewAction::None
    }
}
