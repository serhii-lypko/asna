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

    /// Will be used from the clients
    pub(crate) async fn write_message(&mut self, message: Message) -> io::Result<()> {
        self.write_stream.write_all(&message.encode()).await?;
        self.write_stream.flush().await
    }

    /// Will be used from ingress
    pub(crate) async fn read_message(&mut self) -> crate::Result<Option<Message>> {
        loop {
            // Try parse message first. If no enough data in buffer - try grab some more bytes
            // from tcp stream and try again.
            if let Some(message) = self.try_parse()? {
                return Ok(Some(message));
            }

            // This one is critical.
            // Pending on the client side does not mean read 0.
            // It means write_stream.read_buf will sit and wait for client actions.
            // So basically, it write_stream.read_buf() will return zero when connection
            // is closed (or reset by peer). Otherwise it sits and waits for new bytes to come.
            //
            // So the idea is TCP is low-level stream protocol machinery. It does not provide
            // explicit application-level states. It gives byte-stream behavior plus connection
            // lifecycle semantics. Closure is observed indirectly through EOF.
            //
            // bytes read > 0 -> data arrived
            // future pending -> no data yet, connection still open
            // bytes read == 0 -> peer closed / EOF
            if self.write_stream.read_buf(&mut self.read_buff).await? == 0 {
                if self.read_buff.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn try_parse(&mut self) -> crate::Result<Option<Message>> {
        use crate::message::Error::Incomplete;

        let mut buf = Cursor::new(&self.read_buff[..]);

        match Message::parse(&mut buf) {
            Ok(msg) => {
                // Parsing implictly advanced the cursor position of the buffer
                let parsed_len = buf.position() as usize;
                self.read_buff.advance(parsed_len);

                Ok(Some(msg))
            }

            // There is not enough data present in the read buffer to parse a single message.
            Err(Incomplete) => Ok(None),

            // An error was encountered while parsing the frame.
            Err(e) => unimplemented!(),
        }
    }
}
