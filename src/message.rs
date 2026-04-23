use bytes::{Buf, Bytes};
use std::io::Cursor;
use uuid::Uuid;

pub enum SubmitJobKind {
    Hashing,
}

pub struct SubmitJob {
    id: Uuid,
    kind: SubmitJobKind,

    // Job needs to own it's payload
    payload: Vec<u8>,
}

impl SubmitJob {
    pub fn new(kind: SubmitJobKind, payload: Vec<u8>) -> Self {
        SubmitJob {
            id: Uuid::new_v4(),
            kind,
            payload,
        }
    }
}

pub(crate) enum Message {
    Ping,
    SubmitJob(SubmitJob),
}

impl Message {
    /// Length-prefixed encoding.
    /// [message-kind:1byte][payload-len:4bytes][payload:Nbytes]
    pub(crate) fn encode(&self) -> Vec<u8> {
        let msg_kind = &self.encode_message_kind();

        let payload = "hello".as_bytes();
        let payload_len = payload.len() as u32;

        // Converting payload_len into its big-endian byte representation.
        let payload_len_bytes: &[u8; 4] = &payload_len.to_be_bytes();

        [&msg_kind[..], &payload_len_bytes[..], payload].concat()
    }

    pub(crate) fn parse(src: &mut Cursor<&[u8]>) -> Result<Message, Error> {
        // peek 4-byte length prefix
        // if not enough bytes for prefix, return incomplete
        // if not enough bytes for whole frame, return incomplete
        // otherwise decode one message and consume bytes

        let payload_len = get_u32(src)?;
        dbg!(payload_len);

        todo!()
    }

    fn encode_message_kind(&self) -> [u8; 1] {
        let descriptor: u8 = match self {
            Message::Ping => 0x0,
            Message::SubmitJob(_) => 0x01,
        };

        descriptor.to_be_bytes()
    }
}

/// Consumes exactly 4 bytes from the cursor and returns it as u32.
fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u32())
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
}
