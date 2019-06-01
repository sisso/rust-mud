use std::net::{TcpStream, Shutdown, TcpListener};
use std::io;
use std::io::{Write, BufRead, ErrorKind, Error};

use crate::view_login;
use crate::view_mainloop;

struct Connection {
    id: u32,
    stream: TcpStream,
    pending_inputs: Vec<String>,
    pending_outputs: Vec<String>
}

impl Connection {
    pub fn on_failure(stream: &mut TcpStream, err: Error) {
        println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
        stream.shutdown(Shutdown::Both).unwrap();
    }

    pub fn write(stream: &mut TcpStream, msg: &str) -> io::Result<()> {
        stream.write(msg.as_bytes())?;
        stream.flush()
    }

    pub fn writeln(stream: &mut TcpStream, msg: &str) -> io::Result<()> {
        stream.write(msg.as_bytes())?;
        stream.write("\n".as_bytes())?;
        stream.flush()
    }

    pub fn read_field(stream: &mut TcpStream, field_name: &str) -> io::Result<String> {
        Connection::write(stream, field_name)?;
        Connection::read_line(stream)
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

    pub fn addr(stream: &TcpStream) -> io::Result<String> {
        let a = stream.peer_addr()?;
        let s: String = a.to_string();
        Ok(s)
    }
}

pub struct Server {
    next_conneciton_id: u32,
    tick: u32,
    connections: Vec<Connection>,
    listener: Option<TcpListener>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            next_conneciton_id: 0,
            tick: 0,
            connections: Vec::new(),
            listener: None
        }
    }

    fn next_connection_id(&mut self) -> u32 {
        let id = self.next_conneciton_id;
        self.next_conneciton_id += 1;
        id
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        listener.set_nonblocking(true).expect("non blocking failed");
        // accept connections and process them, spawning a new thread for each one
        println!("Server listening on port 3333");

        self.listener = Some(listener);
    }

    pub fn run(&mut self) {
        let mut broken_connections: Vec<u32> = vec![];

        let listener = self.listener.as_ref().expect("server not started!");

        // accept new connections
        if let Ok((mut stream, addr)) = listener.accept() {
            let id = self.next_connection_id();

            println!("new connection ({}) {}, total connections {}", addr, id, self.connections.len());
            stream.set_nonblocking(true)
                .expect(format!("failed to set non_blocking stream for {}", id).as_str());

            // connection succeeded
            let connection = Connection {
                id: id,
                stream: stream,
                pending_inputs: vec![],
                pending_outputs: vec![],
            };

            self.connections.push(connection);
        }

        // handle inputs
        for connection in &mut self.connections {
            match Connection::read_line(&mut connection.stream) {
                Ok(line) => {
                    connection.pending_inputs.push(line);
                },
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    println!("{} failed: {}", connection.id, e);
                    broken_connections.push(connection.id)
                }
            }
        }

        // handle outputs
        for connection in &mut self.connections {
            for input in &mut connection.pending_outputs {
                Connection::writeln(&mut connection.stream, input.as_str());
            }
        }

        // remove broken connections
        for id in &broken_connections {
            let index = self.connections.iter().position(|i| i.id == *id).unwrap();
            self.connections.remove(index);

            println!("{} removed, total connections {}", *id, self.connections.len());
        }
        broken_connections.clear();
    }
}
