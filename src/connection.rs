use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::message::Message;

/*
    Todo overview

    Receiver needs to know:
    - where one message starts and ends
    - what kind of message it is
    - how to interpret the payload

    On the wire, the receiver first reads enough bytes to know the full message size,
    keeps buffering until that many bytes are available, then parses one complete request.
    Any leftover bytes stay in the buffer for the next message.
*/

pub(crate) struct Connection {
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

    /// Will be used from clients
    pub(crate) async fn write_message(&mut self, message: &Message) -> io::Result<()> {
        self.write_stream.write_all(&message.encode()).await?;
        self.write_stream.flush().await
    }

    /// Will be used from ingress
    pub(crate) async fn read_message(&mut self) -> crate::Result<Option<Message>> {
        loop {
            // Try parse message first. If no enough data in buffer - try grab some more bytes
            // from tcp stream and try again.
            if let Some(message) = self.try_parse()? {
                //
            }

            // 0 indicates end of stream
            if self.write_stream.read_buf(&mut self.read_buff).await? == 0 {
                if self.read_buff.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn try_parse(&self) -> crate::Result<Option<Message>> {
        use crate::message::Error::Incomplete;

        let mut buf = Cursor::new(&self.read_buff[..]);

        match Message::parse(&mut buf) {
            Ok(_) => {
                // dbg!(buf.position());

                // FIXME
                Ok(None)
            }

            // There is not enough data present in the read buffer to parse a single message.
            Err(Incomplete) => Ok(None),

            // An error was encountered while parsing the frame.
            Err(e) => unimplemented!(),
        }
    }
}
