use mud_domain::game::{Game};
use mud_domain::game::container::Container;
use mud_domain::game::loader;
use socket_server::*;
use commons::{TimeTrigger, TotalTime, DeltaTime};
use std::path::Path;
use logs::*;

pub struct ServerRunner {
    server: Box<dyn Server>,
    game: Game,
    save: Option<(String, TimeTrigger)>,
}

impl ServerRunner {
    pub fn new(server: Box<dyn Server>, save: Option<(String, DeltaTime)>) -> Self {
        let mut container: Container = Container::new();
//        loader::load(&mut container);
//        loader::scenery_space::load(&mut container);
        let config_path = Path::new("./data/space");
        info!("loading configuration: {:?}", config_path.canonicalize().unwrap());
        loader::hocon_loader::load(&config_path, &mut container);

        ServerRunner {
            server,
            game: Game::new(container),
            save: save.map(|(file, seconds)| {
                (file, TimeTrigger::new(seconds, TotalTime(0.0)))
            }),
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
            self.game.handle_input(input.connection_id, input.msg.as_ref());
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

pub fn run() {
    let server = server_socket::SocketServer::new();
    let mut game = ServerRunner::new(Box::new(server), Some(("/tmp/current".to_string(), DeltaTime(1.0))));

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        game.run(DeltaTime(0.1));
    }
}
