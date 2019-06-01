use std::net::{TcpStream, Shutdown};
use std::io::{Write, BufRead, ErrorKind};
use std::io::Error;
use std::io;

#[derive(Debug)]
//pub enum Connection {
//    NewConnection {
//        id: u32,
//        stream: TcpStream
//    },
//    PlayerConnection {
//        id: u32,
//        stream: TcpStream,
//        login: String
//    }
//}
//
//pub struct PlayerConnection {
//    pub id: u32,
//    pub login: String,
//    pub connection: Connection,
//}


pub struct Connection {
    pub id: u32,
    pub stream: TcpStream,
    pub login: Option<String>
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
