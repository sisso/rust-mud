use std::fs;
use std::path::{Path, PathBuf};

use commons::DeltaTime;
use logs::*;
use mud_domain::errors::Error;
use mud_domain::game::container::Container;
use mud_domain::game::loader::Loader;
use mud_domain::game::Game;
use mud_domain::game::{loader, GameCfg};
use socket_server::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ServerConfig {
    pub port: u32,
    pub data_folder: PathBuf,
    pub module_path: PathBuf,
    pub profile: Option<String>,
}

pub struct ServerRunner {
    server: Box<dyn Server>,
    server_cfg: ServerConfig,
    game: Game,
    stop_flag: Arc<AtomicBool>,
}

impl ServerRunner {
    pub fn new(
        server: Box<dyn Server>,
        server_cfg: ServerConfig,
        game: Game,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        ServerRunner {
            server,
            server_cfg,
            game,
            stop_flag,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // main loop
        loop {
            std::thread::sleep(::std::time::Duration::from_millis(100));

            let kill_signal = self.stop_flag.load(Ordering::Relaxed);

            self.run_tick(DeltaTime(0.1));

            let tick = self.game.container.time.tick.as_u32();

            // maintenance tasks
            if tick % 100 == 0 || kill_signal {
                // create snapshot
                if self.server_cfg.profile.is_some() {
                    let data = Loader::create_snapshot(&self.game.container)?;

                    let snapshot_file = snapshot_filename(&self.server_cfg, None)?;
                    info!("saving snapshot: {:?}", snapshot_file);
                    Loader::write_snapshot(&snapshot_file, &data)?;

                    let snapshot_history = snapshot_filename(&self.server_cfg, Some(tick))?;
                    info!("saving snapshot: {:?}", snapshot_history);
                    Loader::write_snapshot(&snapshot_history, &data)?;
                }
            }

            if kill_signal {
                info!("receive kill signal, saving and exiting");
                break;
            }
        }

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        unimplemented!()
    }

    fn run_tick(&mut self, delta_time: DeltaTime) {
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

pub fn create_server(server_cfg: ServerConfig, stop_flag: Arc<AtomicBool>) -> Result<ServerRunner> {
    let container = if server_cfg.profile.is_some() {
        setup_profile_folder(&server_cfg)?;
        let profile_file = snapshot_filename(&server_cfg, None)?;
        if profile_file.exists() {
            load_snapshot(&profile_file)?
        } else {
            load_module(&server_cfg)?
        }
    } else {
        load_module(&server_cfg)?
    };

    // create game
    let game_cfg = GameCfg::new();
    let game = Game::new(game_cfg, container);

    // create server
    let server = server_socket::SocketServer::new(server_cfg.port);
    let runner = ServerRunner::new(Box::new(server), server_cfg, game, stop_flag);
    Ok(runner)
}

fn setup_profile_folder(server_cfg: &ServerConfig) -> Result<()> {
    let profile_folder = resolve_profile_path(server_cfg)?;
    fs::create_dir_all(&profile_folder)?;

    if !fs::metadata(&profile_folder)?.is_dir() {
        panic!("profile path at {:?} is not a folder", profile_folder);
    }

    info!("profile folder at {:?}", profile_folder.canonicalize()?);
    Ok(())
}

fn load_snapshot(snapshot_filename: &Path) -> Result<Container> {
    let mut container: Container = Container::new();
    info!("loading from {:?}", snapshot_filename.canonicalize()?);
    let data = Loader::read_snapshot(snapshot_filename)?;
    Loader::load_data(&mut container, data)?;
    Ok(container)
}

fn load_module(server_cfg: &ServerConfig) -> Result<Container> {
    info!(
        "loading configuration: {:?}",
        server_cfg.module_path.canonicalize()?,
    );

    let mut container: Container = Container::new();
    loader::Loader::load_folders(&mut container, &server_cfg.module_path)?;
    Ok(container)
}

fn snapshot_filename(cfg: &ServerConfig, tick: Option<u32>) -> Result<PathBuf> {
    let profile_path = resolve_profile_path(cfg)?;

    let path = match tick {
        Some(tick) => profile_path.join(&format!("snapshot_{}.json", tick)),
        None => profile_path.join("snapshot.json"),
    };

    Ok(path)
}

fn resolve_profile_path(server_cfg: &ServerConfig) -> Result<PathBuf> {
    let profile = server_cfg
        .profile
        .as_ref()
        .ok_or(Error::NotFoundException)?;

    Ok(server_cfg.data_folder.join(profile.as_str()))
}
