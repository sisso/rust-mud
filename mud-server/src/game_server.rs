use std::fs;
use std::path::{PathBuf};

use commons::{DeltaTime};
use logs::*;
use mud_domain::game::container::Container;
use mud_domain::game::loader::Loader;
use mud_domain::game::Game;
use mud_domain::game::{loader, GameCfg};
use socket_server::*;

pub struct ServerConfig {
    pub port: u32,
    pub data_folder: PathBuf,
    pub module_path: PathBuf,
    pub profile: Option<String>,
}

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

pub fn start_server(server_cfg: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_filename = "snapshot.json";

    let path_snapshot_file = if let Some(profile) = server_cfg.profile {
        let profile_folder = server_cfg.data_folder.join(profile.as_str());

        fs::create_dir_all(&profile_folder)?;

        if !fs::metadata(&profile_folder)?.is_dir() {
            panic!("profile path at {:?} is not a folder", profile_folder);
        }

        Some(profile_folder.join(snapshot_filename))
    } else {
        None
    };

    let mut container: Container = Container::new();

    match &path_snapshot_file {
        // if has a snapshot
        Some(path) if path.exists() => {
            info!("loading from {:?}", path.canonicalize()?);
            let data = Loader::read_snapshot(path)?;
            Loader::load_data(&mut container, data)?;
        }

        // profile has no snapshot or no snapshot was provided
        _ => {
            info!(
                "loading configuration: {:?}",
                server_cfg.module_path.canonicalize()?,
            );

            loader::Loader::load_folders(&mut container, &server_cfg.module_path)?;
        }
    }

    let game_cfg = GameCfg::new();
    let game = Game::new(game_cfg, container);

    let server = server_socket::SocketServer::new(server_cfg.port);
    let mut runner = ServerRunner::new(Box::new(server), game);

    loop {
        std::thread::sleep(::std::time::Duration::from_millis(100));
        runner.run(DeltaTime(0.1));

        let tick = runner.game.container.time.tick.as_u32();
        if tick % 100 == 0 {
            if let Some(snapshot_path) = &path_snapshot_file {
                let data = Loader::create_snapshot(&runner.game.container)?;

                info!("saving snapshot: {:?}", snapshot_path.canonicalize()?,);
                Loader::write_snapshot(snapshot_path, &data)?;

                let snapshot_history = snapshot_path
                    .parent()
                    .unwrap()
                    .join(format!("snapshot_{}.json", tick));

                info!("saving snapshot: {:?}", snapshot_history.canonicalize()?,);
                Loader::write_snapshot(&snapshot_history, &data)?;
            }
        }
    }
}
