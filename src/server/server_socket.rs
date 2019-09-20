use super::*;

use std::net::{TcpStream, TcpListener};
use std::io;
use std::io::{Write, BufRead, ErrorKind};

use crate::utils::{ConnectionId, ConnectionOutput};

struct Connection {
    id: ConnectionId,
    stream: TcpStream,
}

pub struct SocketServer {
    next_conneciton_id: u32,
    tick: u32,
    connections: Vec<Connection>,
    listener: Option<TcpListener>,
    pending_outputs: Option<Vec<ConnectionOutput>>,
}

impl Connection {
    pub fn write(stream: &mut TcpStream, msg: &str) -> io::Result<()> {
        stream.write(msg.as_bytes())?;
        stream.flush()
    }

    pub fn read_line(stream: &mut TcpStream) -> io::Result<String> {
        // TODO: how keep this BufReader but dont lose onership of the Stream?
        let mut reader = std::io::BufReader::new(stream);
        let mut buffer = String::new();
        let size = reader.read_line(&mut buffer)?;
        if size == 0 {
            return Err(io::Error::from(ErrorKind::ConnectionAborted));
        }
        let buffer = buffer.trim().to_string();
        Ok(buffer)
    }
}

impl Server for SocketServer {
    fn run(&mut self) -> ServerChanges {
        let outputs = self.pending_outputs.take().unwrap_or(vec![]);
        self.read_write(outputs)
    }

    fn append_output(&mut self, pending_outputs: Vec<ConnectionOutput>) {
        assert!(self.pending_outputs.is_none());
        self.pending_outputs = Some(pending_outputs);
    }
}

impl SocketServer {
    fn next_connection_id(&mut self) -> ConnectionId {
        let id = self.next_conneciton_id;
        self.next_conneciton_id += 1;
        ConnectionId(id)
    }

    pub fn new() -> Self {
        let mut ins = SocketServer {
            next_conneciton_id: 0,
            tick: 0,
            connections: Vec::new(),
            listener: None,
            pending_outputs: None
        };

        ins.start();
        ins
    }

    fn start(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        info!("server - listening on port 3333");

        self.listener = Some(listener);
    }

    fn read_write(&mut self, pending_outputs: Vec<ConnectionOutput>) -> ServerChanges {
        let mut new_connections: Vec<ConnectionId> = vec![];
        let mut broken_connections: Vec<ConnectionId> = vec![];
        let mut pending_inputs: Vec<(ConnectionId, String)> = vec![];

        let listener = self.listener.as_ref().expect("server not started!");

        self.tick += 1;

        // accept new connections
        if let Ok((stream, addr)) = listener.accept() {
            let id = self.next_connection_id();

            info!("server - new connection ({}) {:?}, total connections {}", addr, id, self.connections.len());
            stream.set_nonblocking(true)
                .expect(format!("failed to set non_blocking stream for {:?}", id).as_str());

            // connection succeeded
            let connection = Connection {
                id: id,
                stream: stream,
            };

            new_connections.push(id);
            self.connections.push(connection);
        }

        // handle inputs
        for connection in &mut self.connections {
            match Connection::read_line(&mut connection.stream) {
                Ok(line) => {
                    pending_inputs.push((connection.id, line));
                },
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    warn!("server - {:?} failed: {}", connection.id, e);
                    broken_connections.push(connection.id)
                }
            }
        }

        // handle outputs
        for e in pending_outputs {
            for connection in &mut self.connections {
                let is_dest = e.dest_connections_id.contains(&connection.id);
                if is_dest {
                    debug!("server - {:?} sending '{}'", connection.id, SocketServer::clean_output_to_log(&e.output));

                    if let Err(err) = Connection::write(&mut connection.stream, e.output.as_str()) {
                        warn!("server - {:?} failed: {}", connection.id, err);
                        broken_connections.push(connection.id);
                    }
                }
            }
        }

        // remove broken connections
        for connection in &broken_connections {
            let index = self.connections.iter().position(|i| i.id == *connection).unwrap();
            self.connections.remove(index);

            info!("server - {:?} removed, total connections {}", connection, self.connections.len());
        }

        ServerChanges {
            connects: new_connections,
            disconnects: broken_connections,
            pending_inputs: pending_inputs
        }
    }

    fn clean_output_to_log(s: &String) -> String {
        s.replace("\n", "\\n")
    }
}
