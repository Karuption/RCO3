use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Read;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let con = TcpListener::bind("0.0.0.0:6697")
        .await
        .expect("Cannot open port");

    loop {
        let (socket, addr) = con.accept().await?;
        println!("New connection from {addr:?}");
        tokio::spawn(process(socket));
    }
}

async fn process(socket: TcpStream) -> std::io::Result<()> {
    let mut con = Connection::new(socket);

    while 0 != con.stream.read_buf(&mut con.buff).await? {
        con.stream.write_buf(&mut con.buff).await?;
        con.stream.flush().await?;
    }

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
}
