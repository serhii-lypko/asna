use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

pub struct Connection {
    // Write level buffering
    write_stream: BufWriter<TcpStream>,

    // Reading buffer
    read_buff: BytesMut,
}

impl Connection {
    pub fn new(tcp_stream: TcpStream) -> Self {
        Connection {
            write_stream: BufWriter::new(tcp_stream),

            // 8KB read buffer
            read_buff: BytesMut::with_capacity(8 * 1024),
        }
    }
}
