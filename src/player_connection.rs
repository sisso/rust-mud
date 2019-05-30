use std::net::{TcpStream, Shutdown};
use std::io::{Write, BufRead};
use std::io::Error;
use std::io;

#[derive(Debug)]
pub struct PlayerConnection {
    stream: TcpStream
}

impl PlayerConnection {
    pub fn new(stream: TcpStream) -> PlayerConnection {
        PlayerConnection {
            stream
        }
    }

    pub fn on_failure(&mut self, err: Error) {
        println!("An error occurred, terminating connection with {}", self.stream.peer_addr().unwrap());
        self.stream.shutdown(Shutdown::Both).unwrap();
    }

    pub fn write(&mut self, msg: &str) -> io::Result<()> {
        self.stream.write(msg.as_bytes())?;
        self.stream.flush()
    }

    pub fn writeln(&mut self, msg: &str) -> io::Result<()> {
        self.stream.write(msg.as_bytes())?;
        self.stream.write("\n".as_bytes())?;
        self.stream.flush()
    }

    pub fn read_field(&mut self, field_name: &str) -> io::Result<String> {
        self.write(field_name)?;
        self.read_line()
    }

    pub fn read_line(&mut self) -> io::Result<String> {
        // TODO: how keep this BufReader but dont lose onership of the Stream?
        let mut reader = std::io::BufReader::new(&mut self.stream);
        let mut buffer = String::new();
        let _ = reader.read_line(&mut buffer)?;
        let buffer = buffer.trim().to_string();
        Ok(buffer)
    }

    pub fn addr(&self) -> io::Result<String> {
        let a = self.stream.peer_addr()?;
        let s: String = a.to_string();
        Ok(s)
    }
}
