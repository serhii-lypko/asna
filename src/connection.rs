use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

pub struct Connection {
    //
}

impl Connection {
    pub fn new(tcp_stream: TcpStream) -> Self {
        todo!()
    }
}
