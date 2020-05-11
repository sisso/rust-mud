use commons::{DeltaTime, TimeTrigger, TotalTime};
use logs::*;
use mud_domain::errors::Error;
use mud_domain::game::container::Container;
use mud_domain::game::save::{load_from_file, save_to_file};
use mud_domain::game::Game;
use mud_domain::game::{loader, GameCfg};
use socket_server::*;
use std::path::Path;

pub struct ServerRunner {
    pub server: Box<dyn Server>,
    pub game: Game,
}

impl ServerRunner {
    pub fn new(server: Box<dyn Server>, game: Game) -> Self {
        ServerRunner { server, game }
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

        for (connection_id, msg) in self.game.flush_outputs() {
            self.server.output(connection_id, msg);
        }
    }
}

pub fn start_server(module_path: &str, profile: Option<String>) {
    let config_path = Path::new(module_path);

    info!(
        "loading configuration: {:?}",
        config_path.canonicalize().unwrap()
    );

    let profile_file = profile.map(|profile| format!("/tmp/{}", profile));

    let mut container: Container = Container::new();
    loader::Loader::load_folders(&mut container, &config_path).unwrap();
    if let Some(profile_file) = &profile_file {
        match load_from_file(&mut container, profile_file.as_str()) {
            Ok(_) => {}
            Err(Error::NotFoundFailure) => {}
            Err(other) => panic!(other),
        }
    }

    let cfg = GameCfg::new();
    let game = Game::new(cfg, container);

    let server = server_socket::SocketServer::new();
    let mut runner = ServerRunner::new(Box::new(server), game);

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        runner.run(DeltaTime(0.1));

        if profile_file.is_some() && runner.game.container.time.tick.as_u32() % 100 == 0 {
            save_to_file(
                &runner.game.container,
                profile_file.as_ref().unwrap().as_str(),
            )
            .unwrap();
        }
    }
}
