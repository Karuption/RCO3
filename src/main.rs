mod commands;

use crate::commands::{parse_command, Command};
use bytes::{Buf, BytesMut};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::BufRead;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let con = TcpListener::bind("0.0.0.0:6697")
        .await
        .expect("Cannot open port");

    loop {
        let socket = con.accept().await?;
        println!("{:?} Connected", socket.1);

        tokio::spawn(async move {
            let r = process(socket).await;
            println!("{:?}", r)
        });
    }
}

async fn process(socket: (TcpStream, SocketAddr)) -> Result<(), String> {
    let con = Connection::new(socket.0);
    let mut user = con
        .init(socket.1)
        .await
        .expect("unable to initialize the connection");

    while let Some(cmd) = user.read_command().await.map_err(|x| x.to_string())? {
        println!("{cmd:?}");
    }

    println!("{} has disconnected", user.host_mask());
    Ok(())
}

pub(crate) struct Connection {
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

        let mut lines = self.buff.lines().peekable();

        let mut nick = "".to_string();
        let mut user = "".to_string();
        while let Some(Ok(line)) = lines.next() {
            let cmd = parse_command(line)?;
            println!("{:?}", &cmd);
            match cmd {
                Command::Cap(_) => {}
                Command::Join(channels, _) => {}
                Command::Nick(name, _) => nick = name,
                Command::Quit(msg) => {}
                Command::User(user_input) => user = user_input,
                _ => {}
            }
        }

        self.buff.advance(len);

        let mut user = User::new(
            nick.to_string(),
            user.to_string(),
            addr.ip().to_string(),
            self,
        );

        user.write(b"332 #test :A channel").await?;

        Ok(user)
    }
}
pub struct User {
    nickname: String,
    username: String,
    hostname: String,
    //mode: u32,
    connection: Connection,
}

impl User {
    pub(crate) fn new(
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
        format!("{}!{}@{}", &self.nickname, &self.username, &self.hostname)
    }

    pub async fn write(&mut self, msg: &[u8]) -> io::Result<()> {
        self.connection.stream.write_all(msg).await?;
        self.connection.stream.flush().await
    }

    pub async fn write_command(&mut self, command: Command) -> io::Result<()> {
        self.connection
            .stream
            .write_all(command.write_value()?.as_ref())
            .await?;
        self.connection.stream.flush().await
    }

    pub async fn read_command(&mut self) -> Result<Option<Command>, Box<dyn Error>> {
        if let Some(Ok(raw_cmd)) = self.connection.buff.lines().next() {
            self.connection.buff.advance(raw_cmd.len());
            Ok(Some(commands::parse_command(raw_cmd)?))
        } else {
            let len = self
                .connection
                .stream
                .read_buf(&mut self.connection.buff)
                .await?;
            if len == 0 {
                return Ok(None);
            }

            let raw_cmd = self.connection.buff.lines().next().unwrap()?;
            self.connection.buff.advance(raw_cmd.len());
            Ok(Some(commands::parse_command(raw_cmd)?))
        }
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
