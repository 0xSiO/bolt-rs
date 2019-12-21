use std::convert::TryFrom;
use std::panic::catch_unwind;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::message::chunk::Chunk;
use crate::serialize::{
    DeserializeError, DeserializeResult, Serialize, SerializeError, SerializeResult,
};

mod chunk;
mod init;

struct Message {
    bytes: BytesMut,
}

impl Message {
    pub fn with_capacity(capacity: usize) -> Message {
        Message {
            bytes: BytesMut::with_capacity(capacity),
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.bytes.put(chunk.data);
    }
}

impl Serialize for Message {
    fn get_marker(&self) -> SerializeResult<u8> {
        Err(SerializeError::new("Messages do not have markers"))
    }
}

impl TryFrom<Bytes> for Message {
    type Error = DeserializeError;

    fn try_from(mut bytes: Bytes) -> DeserializeResult<Message> {
        let result = catch_unwind(move || {
            let mut message = Message::with_capacity(bytes.len());
            while bytes.has_remaining() {
                let size: u16 = bytes.get_u16();
                if size == 0 {
                    // We've reached the end of the message
                    break;
                }
                let mut buf = BytesMut::with_capacity(size as usize);
                for _ in 0..size {
                    buf.put_u8(bytes.get_u8());
                }
                debug_assert!(buf.len() == size as usize);
                message.add_chunk(Chunk::try_from(buf.freeze())?)
            }
            Ok(message)
        });
        result
            .unwrap_or(Err(DeserializeError::new(
                "Failed to create Message from Bytes.",
            )))
            .map_err(|err| {
                DeserializeError::new(&format!("Error creating Message from Bytes: {}", err))
            })
    }
}

impl Into<Bytes> for Message {
    fn into(self) -> Bytes {
        self.bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;

    fn new_chunk() -> Chunk {
        Chunk::try_from(Bytes::from_static(&[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ]))
        .unwrap()
    }

    fn new_message() -> Message {
        let mut msg = Message::with_capacity(1);
        msg.add_chunk(new_chunk());
        msg
    }

    #[test]
    fn get_marker() {
        assert!(new_message().get_marker().is_err());
    }

    #[test]
    fn into_bytes() {
        let bytes: Bytes = new_message().into();
        assert_eq!(bytes, new_chunk().data)
    }

    #[test]
    fn into_bytes_multiple_chunks {
        todo!();
    }

    #[test]
    fn from_bytes() {
        let bytes = Bytes::from_static(&[
            0x00, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x00,
        ]);
        let message = Message::try_from(bytes);
        assert!(message.is_ok());
        let message_bytes: Bytes = message.unwrap().into();
        assert_eq!(message_bytes, new_chunk().data);
    }

    #[test]
    fn from_bytes_multiple_chunks() {
        let bytes = Bytes::from_static(&[
            0x00, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04, 0x00, 0x00,
        ]);
        let message = Message::try_from(bytes);
        assert!(message.is_ok());
        let message_bytes: Bytes = message.unwrap().into();
        assert_eq!(
            message_bytes,
            Bytes::from_static(&[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F, 0x01, 0x02, 0x03, 0x04
            ])
        );
    }
}
