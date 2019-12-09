use commons::{DeltaTime, TimeTrigger, TotalTime};
use logs::*;
use mud_domain::game::container::Container;
use mud_domain::game::loader;
use mud_domain::game::Game;
use socket_server::*;
use std::path::Path;

pub struct ServerRunner {
    server: Box<dyn Server>,
    game: Game,
    save: Option<(String, TimeTrigger)>,
}

impl ServerRunner {
    pub fn new(server: Box<dyn Server>, container: Container, save: Option<(String, DeltaTime)>) -> Self {
       ServerRunner {
            server,
            game: Game::new(container),
            save: save.map(|(file, seconds)| (file, TimeTrigger::new(seconds, TotalTime(0.0)))),
        }
    }

    pub fn run(&mut self, delta_time: DeltaTime) {
        let result = self.server.run();

        for connection_id in result.connects {
            self.game.add_connection(connection_id);
        }

        for connection_id in result.disconnects {
            self.game.disconnect(connection_id);
        }

        for input in result.inputs {
            self.game
                .handle_input(input.connection_id, input.msg.as_ref());
        }

        self.game.tick(delta_time);

        for (connection_id, msg) in self.game.get_outputs() {
            self.server.output(connection_id, msg);
        }

        //        if let Some((save_file, trigger)) = self.save.as_mut() {
        //            if trigger.check(self.game_time.total) {
        //                let save_file = format!("{}_{}.jsonp", save_file, self.game_time.tick.0);
        //                let mut save = SaveToFile::new(save_file.as_ref());
        //                self.game.save(&mut save);
        //                save.close()
        //            }
        //        }
    }
}

pub fn run(module_path: &str) {
    let config_path = Path::new(module_path);

    info!(
        "loading configuration: {:?}",
        config_path.canonicalize().unwrap()
    );

    let mut container: Container = Container::new();
    loader::Loader::load_folder(&mut container, &config_path).unwrap();

    let server = server_socket::SocketServer::new();
    let mut runner = ServerRunner::new(
        Box::new(server),
        container,
        Some(("/tmp/current".to_string(), DeltaTime(1.0))),
    );

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        runner.run(DeltaTime(0.1));
    }
}
