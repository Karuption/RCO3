mod commands;

use crate::commands::{parse_command, Command};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{BufRead, Read};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let con = TcpListener::bind("0.0.0.0:6697")
        .await
        .expect("Cannot open port");

    loop {
        let socket = con.accept().await?;
        println!("{:?} Connected", socket.1);

        tokio::spawn(async move { process(socket).await });
    }
}

async fn process(socket: (TcpStream, SocketAddr)) -> io::Result<()> {
    let mut con = Connection::new(socket.0);
    let mut user = con
        .init(socket.1)
        .await
        .expect("unable to initialize the connection");

    Ok(())
}

struct Connection {
    stream: BufWriter<TcpStream>,
    buff: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(socket),
            buff: BytesMut::with_capacity(1024),
        }
    }

    pub async fn init(mut self, addr: SocketAddr) -> Result<User, Box<dyn Error>> {
        let len = self.stream.read_buf(&mut self.buff).await?;
        if len == 0 {
            return Err("buffer closed before init".into());
        }

        println!("{:?}", &self.buff);

        let mut lines = self.buff.lines().peekable();

        let mut nick = "".to_string();
        let mut user = "".to_string();
        while let Some(Ok(line)) = lines.next() {
            let cmd = parse_command(line)?;
            println!("{:?}", &cmd);
            match cmd {
                Command::CAP(_) => {}
                Command::Join(channels, _) => {}
                Command::Nick(name, _) => nick = name,
                Command::Quit(msg) => {}
                Command::User(user_input) => user = user_input,
                _ => {}
            }
        }

        self.buff.advance(len);

        Ok(User::new(
            nick.to_string(),
            user.to_string(),
            addr.ip().to_string(),
            self,
        ))
    }
}
struct User {
    nickname: String,
    username: String,
    hostname: String,
    //mode: u32,
    connection: Connection,
}

impl User {
    pub fn new(
        nickname: String,
        username: String,
        hostname: String,
        connection: Connection,
    ) -> Self {
        Self {
            nickname,
            username,
            hostname,
            connection,
        }
    }

    pub fn host_mask(&self) -> String {
        format!("{}!{}@{}", self.nickname, self.username, self.hostname)
    }
}

impl Debug for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User {{ nick: {:?} }}, Username: {{ username: {:?} }}",
            self.nickname, self.username
        )
    }
}
