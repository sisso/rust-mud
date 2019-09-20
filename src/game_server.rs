use crate::game::{Game, RunnerParams};
use crate::game::container::Container;
use crate::game::domain::GameTime;
use crate::game::loader;
use crate::server;
use crate::utils::*;
use crate::utils::save::*;

pub struct ServerRunner {
    server: Box<dyn server::Server>,
    game_time: GameTime,
    game: Game,
    save: Option<(String, TimeTrigger)>,
}

impl ServerRunner {
    pub fn new(server: Box<dyn server::Server>, save: Option<(String, Second)>) -> Self {
        let mut container: Container = Container::new();
        loader::load(&mut container);

        ServerRunner {
            server,
            game_time: GameTime {
                tick: Tick(0),
                total: Second(0.0),
                delta: Second(0.1)
            },
            game: Game::new(container),
            save: save.map(|(file, seconds)| {
                (file, TimeTrigger::new(seconds, Second(0.0)))
            }),
        }
    }

    pub fn run(&mut self, delta: Second) {
        self.game_time.tick  = self.game_time.tick.next();
        self.game_time.total = self.game_time.total + delta;
        self.game_time.delta = delta;

        let result = self.server.run();

        let params = RunnerParams {
            connects: result.connects,
            disconnects: result.disconnects,
            inputs: result.pending_inputs,
        };

        let outputs = self.game.handle(self.game_time, params);
        self.server.append_output(outputs);

        if let Some((save_file, trigger)) = self.save.as_mut() {
            if trigger.check(self.game_time.total) {
                let save_file = format!("{}_{}.jsonp", save_file, self.game_time.tick.0);
                let mut save = SaveToFile::new(save_file.as_ref());
                self.game.save(&mut save);
                save.close()
            }
        }
    }
}

pub fn run() {
    let server = server::server_socket::SocketServer::new();
    let mut game = ServerRunner::new(Box::new(server), Some(("/tmp/current".to_string(), Second(1.0))));

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        game.run(Second(0.1));
    }
}
