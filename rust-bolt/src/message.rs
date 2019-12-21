use std::mem;

use bytes::{BufMut, Bytes, BytesMut};

use crate::message::chunk::Chunk;
use crate::serialize::{Serialize, SerializeError, SerializeResult};

mod chunk;
mod init;

struct Message {
    chunks: Vec<Chunk>,
}

impl Serialize for Message {
    fn get_marker(&self) -> SerializeResult<u8> {
        Err(SerializeError::new("Messages do not have markers"))
    }
}

impl Into<Bytes> for Message {
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(
            // Hard to find a "good" worst-case size here
            mem::size_of::<Chunk>() * self.chunks.len() + mem::size_of::<u8>() * 2,
        );
        for chunk in self.chunks {
            let chunk_bytes: Bytes = chunk.into();
            bytes.put(chunk_bytes);
        }
        bytes.put_u16(0);
        bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;

    fn new_message() -> Message {
        Message {
            chunks: vec![Chunk::try_from(Bytes::from_static(&[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F,
            ]))
            .unwrap()],
        }
    }

    #[test]
    fn get_marker() {
        assert!(new_message().get_marker().is_err());
    }

    #[test]
    fn into_bytes() {
        let bytes: Bytes = new_message().into();
        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0x00, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
                0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x00
            ])
        )
    }
}
