use bytes::{Buf, Bytes};
use std::io::Cursor;
use uuid::Uuid;

#[derive(Debug)]
pub enum SubmitJobKind {
    Hashing,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub(crate) enum Message {
    Ping,
    Pong,

    SubmitJob(SubmitJob),
}

impl Message {
    /// Length-prefixed encoding.
    /// [message-kind:1byte][payload-len:4bytes][payload:Nbytes]
    pub(crate) fn encode(&self) -> Vec<u8> {
        let msg_kind = &self.encode_msg_kind();

        match self {
            Message::Ping => msg_kind.into(),
            Message::Pong => msg_kind.into(),
            Message::SubmitJob(submit_job) => {
                let payload = "hello".as_bytes();
                let payload_len = payload.len() as u32;

                // Converting payload_len into its big-endian byte representation.
                let payload_len_bytes: &[u8; 4] = &payload_len.to_be_bytes();

                [&msg_kind[..], &payload_len_bytes[..], payload].concat()
            }
        }
    }

    pub(crate) fn parse(src: &mut Cursor<&[u8]>) -> Result<Message, Error> {
        let msg_kind = Message::decode_msg_kind(get_u8(src)?)?;

        match msg_kind {
            Message::Ping => Ok(Message::Ping),
            Message::Pong => Ok(Message::Pong),
            Message::SubmitJob(submit_job) => {
                let payload_len = get_u32(src)?;

                todo!()
            }
        }
    }

    fn encode_msg_kind(&self) -> [u8; 1] {
        let descriptor: u8 = match self {
            Message::Ping => 0x0,
            Message::Pong => 0x1,
            Message::SubmitJob(_) => 0x02,
        };

        descriptor.to_be_bytes()
    }

    fn decode_msg_kind(descriptor: u8) -> Result<Self, Error> {
        match descriptor {
            0x0 => Ok(Message::Ping),
            0x1 => Ok(Message::Pong),
            _ => unimplemented!(),
        }
    }
}

/// Consumes exactly 1 byte from the cursor and returns it as u8.
fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
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
