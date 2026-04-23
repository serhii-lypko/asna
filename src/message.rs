use bytes::{Buf, Bytes};
use std::io::Cursor;
use uuid::Uuid;

// #[derive(Debug)]
// pub enum JobKind {
//     Hashing,
// }

#[derive(Debug)]
pub struct SubmitJob {
    id: Uuid,

    // TODO
    // kind: JobKind,

    // Job needs to own it's payload
    payload: Vec<u8>,
}

impl SubmitJob {
    pub fn new(payload: Vec<u8>) -> Self {
        SubmitJob {
            id: Uuid::new_v4(),
            payload,
        }
    }
}

// Wire-level tag enum
enum MessageKind {
    Ping,
    Pong,
    SubmitJob,
}

#[derive(Debug)]
pub(crate) enum Message {
    Ping,
    Pong,

    SubmitJob(SubmitJob),
}

impl Message {
    /// Wire format note:
    /// `Ping` and `Pong` are encoded as a single kind byte only, since they carry no payload.
    /// `SubmitJob` is encoded as `[kind:1][payload_len:4][payload:N]`.
    /// So the protocol is not uniformly length-prefixed for every message; only
    /// payload-bearing messages include the explicit length field.
    pub(crate) fn encode(self) -> Vec<u8> {
        let msg_kind = &self.encode_msg_kind();

        match self {
            Message::Ping => msg_kind.into(),
            Message::Pong => msg_kind.into(),
            Message::SubmitJob(submit_job) => {
                // NOTE: skipping JobKind for now.

                let payload = submit_job.payload;
                let payload_len = payload.len() as u32;

                // Converting payload_len into its big-endian byte representation.
                let payload_len_bytes: &[u8; 4] = &payload_len.to_be_bytes();

                [&msg_kind[..], &payload_len_bytes[..], &payload].concat()
            }
        }
    }

    pub(crate) fn parse(src: &mut Cursor<&[u8]>) -> Result<Message, Error> {
        let msg_kind = Message::decode_msg_kind(get_u8(src)?)?;

        match msg_kind {
            MessageKind::Ping => Ok(Message::Ping),
            MessageKind::Pong => Ok(Message::Pong),
            MessageKind::SubmitJob => {
                // NOTE: skipping JobKind for now.

                let payload_len = get_u32(src)? as usize;
                let payload = get_bytes(src, payload_len)?;

                let job = SubmitJob::new(payload.into());
                Ok(Message::SubmitJob(job))
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

    fn decode_msg_kind(descriptor: u8) -> Result<MessageKind, Error> {
        match descriptor {
            0x0 => Ok(MessageKind::Ping),
            0x1 => Ok(MessageKind::Pong),
            0x2 => Ok(MessageKind::SubmitJob),
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

fn get_bytes<'a>(src: &mut Cursor<&'a [u8]>, n: usize) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = start + n;

    // `get_ref()` returns the full underlying byte slice the cursor is reading from.
    // The cursor only tracks the current position; the bytes still live in that slice.
    if src.get_ref().len() < end {
        return Err(Error::Incomplete);
    }

    src.set_position(end as u64);
    Ok(&src.get_ref()[start..end])
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
}
