use crate::stream::Stream;
use std::io::Result;
use std::net::{TcpStream, ToSocketAddrs};

pub struct Client<T: Stream = TcpStream> {
    stream: T,
    pub nick: String,
}

impl Client<TcpStream> {
    pub fn connect<U: ToSocketAddrs>(arg: U, nick: String) -> Result<Self> {
        let client = Client {
            stream: TcpStream::connect(arg)?,
            nick,
        };

        //client.stream.write(format!("NICK {n}\r\n", n = nick).as_bytes())?;
        //client.stream.write(format!("USER {n} 0 * :Ronnie Reagan\r\n", n = nick).as_bytes())?;

        Ok(client)
    }
}
