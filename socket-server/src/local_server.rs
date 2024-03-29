use super::*;

use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

use commons::ConnectionId;
use std::sync::atomic::{AtomicBool, Ordering};

struct AsyncStdIn {
    close: Arc<AtomicBool>,
    buffer: Arc<Mutex<Vec<String>>>,
}

impl AsyncStdIn {
    pub fn new() -> Self {
        let instance = AsyncStdIn {
            close: Arc::new(AtomicBool::new(false)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        };

        let thread_buffer = instance.buffer.clone();
        let thread_close = instance.close.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut string = String::new();
                let _ = stdin.read_line(&mut string);
                let mut buffer = thread_buffer.lock().unwrap();
                buffer.push(string);

                let is_close: bool = thread_close.load(Ordering::Relaxed);
                if is_close {
                    break;
                }
            }
        });

        instance
    }

    pub fn take(&mut self) -> Vec<String> {
        let mut buffer = self.buffer.lock().unwrap();
        std::mem::replace(&mut *buffer, Vec::new())
    }

    pub fn close(&mut self) {
        self.close.swap(true, Ordering::Relaxed);
    }
}

pub struct LocalServer {
    return_connected: bool,
    asyncsdin: AsyncStdIn,
}

impl LocalServer {
    pub fn new() -> Self {
        LocalServer {
            return_connected: false,
            asyncsdin: AsyncStdIn::new(),
        }
    }
}

impl SocketServer for LocalServer {
    fn run(&mut self) -> ServerChanges {
        let mut sc = ServerChanges {
            connects: vec![],
            disconnects: vec![],
            inputs: vec![],
        };

        if self.return_connected {
            for msg in self.asyncsdin.take() {
                sc.inputs.push(ServerInput {
                    connection_id: ConnectionId(0),
                    msg,
                });
            }
        } else {
            sc.connects.push(ConnectionId(0));
            self.return_connected = true;
        }

        sc
    }

    fn output(&mut self, _connection_id: ConnectionId, msg: String) {
        print!("{}", msg);
    }

    fn disconnect(&mut self, _connection_id: ConnectionId) {
        log::info!("DISCONNECT!");
        self.asyncsdin.close();
    }
}
