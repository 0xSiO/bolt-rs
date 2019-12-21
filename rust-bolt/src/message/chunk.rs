use std::convert::TryFrom;
use std::mem;

use bytes::{BufMut, Bytes, BytesMut};

use crate::serialize::{DeserializeError, Serialize, SerializeError, SerializeResult};

pub struct Chunk {
    size: u16,
    data: Bytes,
}

impl Serialize for Chunk {
    fn get_marker(&self) -> SerializeResult<u8> {
        Err(SerializeError::new("Chunks do not have markers"))
    }
}

impl TryFrom<Bytes> for Chunk {
    type Error = DeserializeError;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        if bytes.len() > std::u16::MAX as usize {
            return Err(DeserializeError::new(&format!(
                "Bytes length too long: {}",
                bytes.len()
            )));
        }

        Ok(Chunk {
            size: bytes.len() as u16,
            data: bytes,
        })
    }
}

impl Into<Bytes> for Chunk {
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(
            // 16-bit size, chunk data, then two 0 bytes
            mem::size_of::<u8>() * 2 + self.data.len() + mem::size_of::<u8>() * 2,
        );
        bytes.put_u16(self.size);
        bytes.put(self.data);
        bytes.freeze()
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

    #[test]
    fn get_marker() {
        assert!(new_chunk().get_marker().is_err());
    }
}
