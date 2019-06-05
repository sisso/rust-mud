use std::net::{TcpStream, TcpListener};
use std::io;
use std::io::{Write, BufRead, ErrorKind};

struct Connection {
    id: u32,
    stream: TcpStream,
}

pub struct LoopResult {
    pub connects: Vec<u32>,
    pub disconnects: Vec<u32>,
    pub pending_inputs: Vec<(u32, String)>,
}

pub struct Server {
    next_conneciton_id: u32,
    tick: u32,
    connections: Vec<Connection>,
    listener: Option<TcpListener>,
}

pub struct Output {
    pub dest_connections_id: Vec<u32>,
    pub output: String
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

impl Server {
    fn next_connection_id(&mut self) -> u32 {
        let id = self.next_conneciton_id;
        self.next_conneciton_id += 1;
        id
    }

    pub fn new() -> Self {
        Server {
            next_conneciton_id: 0,
            tick: 0,
            connections: Vec::new(),
            listener: None,
        }
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        println!("server - listening on port 3333");

        self.listener = Some(listener);
    }

    pub fn run(&mut self, pending_outputs: Vec<Output>) -> LoopResult {
        let mut new_connections: Vec<u32> = vec![];
        let mut broken_connections: Vec<u32> = vec![];
        let mut pending_inputs: Vec<(u32, String)> = vec![];

        let listener = self.listener.as_ref().expect("server not started!");

        self.tick += 1;

        // accept new connections
        if let Ok((stream, addr)) = listener.accept() {
            let id = self.next_connection_id();

            println!("server - new connection ({}) {}, total connections {}", addr, id, self.connections.len());
            stream.set_nonblocking(true)
                .expect(format!("failed to set non_blocking stream for {}", id).as_str());

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
                    println!("server - {} failed: {}", connection.id, e);
                    broken_connections.push(connection.id)
                }
            }
        }

        // handle outputs
        for e in pending_outputs {
            for connection in &mut self.connections {
                let is_dest = e.dest_connections_id.contains(&connection.id);
                if is_dest {
                    println!("server - {} sending '{}'", connection.id, e.output);

                    if let Err(err) = Connection::write(&mut connection.stream, e.output.as_str()) {
                        println!("server - {} failed: {}", connection.id, err);
                        broken_connections.push(connection.id);
                    }
                }
            }
        }

        // remove broken connections
        for id in &broken_connections {
            let index = self.connections.iter().position(|i| i.id == *id).unwrap();
            self.connections.remove(index);

            println!("server - {} removed, total connections {}", *id, self.connections.len());
        }

        LoopResult {
            connects: new_connections,
            disconnects: broken_connections,
            pending_inputs: pending_inputs
        }
    }
}
