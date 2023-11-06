mod commands;

use crate::commands::{parse_command, Command};
use bytes::{Buf, BytesMut};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::io::{self, BufRead};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let con = TcpListener::bind("192.168.1.168:6697")
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

    loop {
        let cmd_result = user.read_command().await.map_err(|x| x.to_string());
        if let Err(e) = cmd_result {
            //handle unknown command
            println!("Error parsing command: {e:?}");
            continue;
        }
        if let Ok(None) = cmd_result {
            continue;
        }

        println!("{cmd_result:?}");
        match cmd_result?.unwrap() {
            Command::Cap(_) => {}
            Command::Join(_channels, _) => {}
            Command::List(_) => {}
            Command::Names(_) => {}
            Command::Nick(nick, _) => user.nickname = nick,
            Command::Ping => {}
            Command::Pong => {}
            Command::Topic(_) => {}
            Command::User(_) => {}
            Command::Quit(_q) => {
                println!("User quitting {:?}", _q.get_msg()); //broadcast leaving msg
                break;
            }
        }
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
                Command::Nick(name, _) => nick = name,
                Command::Quit(_msg) => {}
                Command::User(user_input) => user = user_input.user,
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
            self.connection.buff.advance(raw_cmd.len() + 2);

            println!("Raw Command: {:?}", raw_cmd);

            return Ok(Some(commands::parse_command(raw_cmd)?));
        }

        let len = self
            .connection
            .stream
            .read_buf(&mut self.connection.buff)
            .await?;
        if len == 0 {
            return Ok(None);
        }

        let raw_cmd = self.connection.buff.lines().next().unwrap()?;

        //advancing the buffer since making an iterator doesn't move the cursor
        //adding 2 bytes to the cursor advancement account for the line return on windows
        println!("Raw Command: {:?}", raw_cmd);
        self.connection.buff.advance(raw_cmd.len() + 2);

        Ok(Some(commands::parse_command(raw_cmd)?))
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

impl Drop for User {
    fn drop(&mut self) {
        println!("User dropped: {:?}", self.connection.buff);
    }
}
