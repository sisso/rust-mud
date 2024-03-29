use std::io;
use std::io::{BufRead, ErrorKind, Write};
use std::net::{TcpListener, TcpStream};

use super::*;
use commons::ConnectionId;
use std::collections::HashMap;

pub struct DefaultSocketServer {
    next_connection_id: u32,
    connections: HashMap<ConnectionId, Connection>,
    listener: Option<TcpListener>,
    pending_outputs: Option<Vec<ServerOutput>>,
    port: u32,
}

struct Connection {
    id: ConnectionId,
    stream: TcpStream,
}

impl Connection {
    pub fn write(&mut self, msg: &str) -> io::Result<()> {
        self.stream.write(msg.as_bytes())?;
        self.stream.flush()
    }

    pub fn read_line(&mut self) -> io::Result<String> {
        // TODO: how keep this BufReader but don't lose ownership of the Stream?
        let mut reader = std::io::BufReader::new(&mut self.stream);
        let mut buffer = String::new();
        let size = reader.read_line(&mut buffer)?;
        if size == 0 {
            return Err(io::Error::from(ErrorKind::ConnectionAborted));
        }
        let buffer = buffer.trim().to_string();
        Ok(buffer)
    }
}

impl SocketServer for DefaultSocketServer {
    fn run(&mut self) -> ServerChanges {
        let outputs = self.pending_outputs.take().unwrap_or(vec![]);
        self.read_write(outputs)
    }

    fn output(&mut self, connection_id: ConnectionId, msg: String) {
        self.pending_outputs
            .get_or_insert(vec![])
            .push(ServerOutput { connection_id, msg });
    }

    fn disconnect(&mut self, _connection_id: ConnectionId) {
        unimplemented!()
    }
}

impl DefaultSocketServer {
    fn next_connection_id(&mut self) -> ConnectionId {
        let id = self.next_connection_id;
        self.next_connection_id += 1;
        ConnectionId(id)
    }

    pub fn new(port: u32) -> Self {
        let mut ins = DefaultSocketServer {
            next_connection_id: 0,
            connections: HashMap::new(),
            listener: None,
            pending_outputs: None,
            port,
        };

        ins.start();
        ins
    }

    fn start(&mut self) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        log::info!("server - listening on port 3333");

        self.listener = Some(listener);
    }

    fn read_write(&mut self, pending_outputs: Vec<ServerOutput>) -> ServerChanges {
        let mut connects: Vec<ConnectionId> = vec![];
        let mut disconnects: Vec<ConnectionId> = vec![];
        let mut pending_inputs: Vec<ServerInput> = vec![];

        let listener = self.listener.as_ref().expect("server not started!");

        // accept new connections
        if let Ok((stream, addr)) = listener.accept() {
            let id = self.next_connection_id();

            log::info!(
                "new connection ({}) {:?}, total connections {}",
                addr,
                id,
                self.connections.len()
            );
            stream
                .set_nonblocking(true)
                .expect(format!("failed to set non_blocking stream for {:?}", id).as_str());

            // connection succeeded
            let connection = Connection { id, stream };

            connects.push(id);
            self.connections.insert(connection.id, connection);
        }

        // handle inputs
        for (connection_id, stream) in &mut self.connections {
            match stream.read_line() {
                Ok(msg) => pending_inputs.push(ServerInput {
                    connection_id: *connection_id,
                    msg,
                }),
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    log::warn!("{:?} failed: {}", connection_id, e);
                    disconnects.push(*connection_id)
                }
            }
        }

        // handle outputs
        for output in pending_outputs {
            log::trace!("{:?} sending '{:?}'", output.connection_id, &output.msg);

            match self.connections.get_mut(&output.connection_id) {
                Some(connection) => {
                    if let Err(err) = connection.write(output.msg.as_str()) {
                        log::warn!("{:?} failed: {}", connection.id, err);
                        disconnects.push(connection.id);
                    }
                }
                None => log::error!("{:?} not found", output.connection_id),
            }
        }

        // remove broken connections
        for connection in &disconnects {
            self.connections.remove(connection);

            log::info!(
                "{:?} removed, total connections {}",
                connection,
                self.connections.len()
            );
        }

        ServerChanges {
            connects,
            disconnects,
            inputs: pending_inputs,
        }
    }
}
