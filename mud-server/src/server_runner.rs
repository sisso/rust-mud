use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::http_handler;
use commons::DeltaTime;
use http_server::HttpServer;

use mud_domain::*;
use socket_server::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const TICKS_SNAPSHOT: u32 = 1000;

pub struct ServerConfig {
    pub socket_port: u32,
    pub http_port: u32,
    pub data_folder: PathBuf,
    pub module_path: PathBuf,
    pub profile: Option<String>,
}

pub struct ServerRunner {
    server_cfg: ServerConfig,
    socket_server: Box<dyn SocketServer>,
    http_server: Box<dyn HttpServer>,
    game: Game,
    stop_flag: Arc<AtomicBool>,
}

impl ServerRunner {
    pub fn new(
        server_cfg: ServerConfig,
        socket_server: Box<dyn SocketServer>,
        http_server: Box<dyn HttpServer>,
        game: Game,
        stop_flag: Arc<AtomicBool>,
    ) -> Self {
        ServerRunner {
            server_cfg,
            socket_server,
            http_server,
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
            if tick % TICKS_SNAPSHOT == 0 || kill_signal {
                // create snapshot
                if self.server_cfg.profile.is_some() {
                    self.save_snapshot(tick).unwrap();
                }
            }

            if kill_signal {
                log::info!("receive kill signal, saving and exiting");
                break;
            }
        }

        Ok(())
    }

    fn save_snapshot(&mut self, tick: u32) -> Result<()> {
        self.write_loader_snapshot(tick)?;
        // self.write_container_snapshot(tick)?; // @see create_container
        Ok(())
    }

    fn write_loader_snapshot(&mut self, tick: u32) -> Result<()> {
        let data = Loader::create_snapshot(&self.game.container)?;

        let snapshot_file = snapshot_loader_filepath(&self.server_cfg, None)?;
        log::info!(
            "backup snapshot: {:?}",
            snapshot_file.with_file_name("snapshot_backup.json")
        );
        let _ = backup_filename(snapshot_file.as_path()).map_err(|err| {
            log::warn!("fail to generate backup: {:?}", err);
        });
        log::info!("saving snapshot: {:?}", snapshot_file);
        Loader::write_snapshot(&snapshot_file, &data)?;

        let snapshot_history_path = snapshot_loader_filepath(&self.server_cfg, Some(tick))?;
        log::info!("saving snapshot: {:?}", snapshot_history_path);
        Loader::write_snapshot(&snapshot_history_path, &data)?;
        Ok(())
    }

    fn write_container_snapshot(&mut self, tick: u32) -> Result<()> {
        let file_path_tick = snapshot_container_filepath(&self.server_cfg, Some(tick))?;

        log::info!("saving container {:?}", file_path_tick);
        let mut file = fs::File::create(&file_path_tick)?;
        serde_json::to_writer_pretty(&mut file, &self.game.container)?;

        let file_path = snapshot_container_filepath(&self.server_cfg, None)?;
        log::info!(
            "copy container snapshot {:?} to main file {:?}",
            file_path_tick,
            file_path
        );
        fs::copy(&file_path_tick, &file_path)?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        unimplemented!()
    }

    fn run_tick(&mut self, delta_time: DeltaTime) {
        // handle socket requests
        let socket_requests = self.socket_server.run();

        for connection_id in socket_requests.connects {
            self.game.add_connection(connection_id);
        }

        for connection_id in socket_requests.disconnects {
            self.game.disconnect(connection_id);
        }

        for input in socket_requests.inputs {
            self.game
                .handle_input(input.connection_id, input.msg.as_ref());
        }

        // handle http requests
        let http_requests = self
            .http_server
            .take_requests()
            .expect("fail to take http requests");
        let http_responses = http_handler::handle_requests(&mut self.game, http_requests);
        self.http_server
            .provide_responses(http_responses)
            .expect("fail to provide responses");

        // update game
        self.game.tick(delta_time);

        // sockets responses
        for (connection_id, msg) in self.game.flush_outputs() {
            self.socket_server.output(connection_id, msg);
        }
    }
}

pub fn create_server(server_cfg: ServerConfig, stop_flag: Arc<AtomicBool>) -> Result<ServerRunner> {
    let container = create_container(&server_cfg)?;

    // create game
    let game_cfg = GameCfg::new();
    let game = Game::new(game_cfg, container);

    // create server
    let socket_server = server_socket::DefaultSocketServer::new(server_cfg.socket_port);
    let http_server = <dyn http_server::HttpServer>::new(server_cfg.http_port)
        .expect("fail to create http server");
    let runner = ServerRunner::new(
        server_cfg,
        Box::new(socket_server),
        http_server,
        game,
        stop_flag,
    );
    Ok(runner)
}

fn create_container(server_cfg: &ServerConfig) -> Result<Container> {
    if server_cfg.profile.is_some() {
        setup_profile_folder(&server_cfg)?;
        // we prefer to use loader over container serialization as load is more compatible with
        // others services like in-game admin and the rest api.
        let profile_file = snapshot_loader_filepath(&server_cfg, None)?;
        if profile_file.exists() {
            log::info!(
                "profile progress found at {}, loading",
                profile_file.canonicalize().unwrap().to_str().unwrap()
            );
            load_snapshot(&profile_file)
        } else {
            log::info!("profile has no progress, loading from configuration");
            load_module(&server_cfg)
        }
    } else {
        log::info!("not profile defined, loading from configuration");
        load_module(&server_cfg)
    }
}

fn setup_profile_folder(server_cfg: &ServerConfig) -> Result<()> {
    let profile_folder = resolve_profile_path(server_cfg)?;
    fs::create_dir_all(&profile_folder)?;

    if !fs::metadata(&profile_folder)?.is_dir() {
        panic!("profile path at {:?} is not a folder", profile_folder);
    }

    log::info!("profile folder at {:?}", profile_folder.canonicalize()?);
    Ok(())
}

fn load_container_snapshot(container_snapshot_file: &Path) -> Result<Container> {
    log::info!(
        "loading container from {:?}",
        container_snapshot_file.canonicalize()?
    );
    let file = File::open(container_snapshot_file)?;
    let container: Container = serde_json::from_reader(file)?;
    Ok(container)
}

fn load_snapshot(snapshot_filename: &Path) -> Result<Container> {
    let mut container: Container = Container::new();
    log::info!("loading from {:?}", snapshot_filename.canonicalize()?);
    let data = Loader::read_snapshot(snapshot_filename)?;
    Loader::load_data(&mut container, data)?;
    Ok(container)
}

fn load_module(server_cfg: &ServerConfig) -> Result<Container> {
    log::info!(
        "loading configuration: {:?}",
        server_cfg.module_path.canonicalize()?,
    );

    let mut container: Container = Container::new();
    Loader::load_folders(&mut container, server_cfg.module_path.as_path())?;
    Ok(container)
}

fn backup_filename(path: &Path) -> Result<()> {
    let new_path = path.with_file_name("snapshot_backup.json");
    std::fs::rename(path, new_path)?;
    Ok(())
}

fn snapshot_loader_filepath(cfg: &ServerConfig, tick: Option<u32>) -> Result<PathBuf> {
    let profile_path = resolve_profile_path(cfg)?;

    let path = match tick {
        Some(tick) => profile_path.join(&format!("snapshot_{}.json", tick)),
        None => profile_path.join("save.json"),
    };

    Ok(path)
}

fn snapshot_container_filepath(cfg: &ServerConfig, tick: Option<u32>) -> Result<PathBuf> {
    let profile_path = resolve_profile_path(cfg)?;

    let path = match tick {
        Some(tick) => profile_path.join(&format!("snapshot_container_{}.json", tick)),
        None => profile_path.join("save_container.json"),
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
