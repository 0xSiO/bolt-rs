use std::convert::{TryFrom, TryInto};
use std::mem;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::io::BufStream;
use tokio::prelude::*;

use crate::error::*;
use crate::message::Chunk;
use crate::serialization::*;

#[derive(Debug)]
pub(crate) struct MessageBytes {
    bytes: BytesMut,
}

impl MessageBytes {
    pub(crate) fn new() -> MessageBytes {
        MessageBytes {
            bytes: BytesMut::new(),
        }
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        self.bytes.put(chunk.data);
    }

    pub(crate) async fn from_stream<T: Unpin + AsyncRead + AsyncWrite>(
        buf_stream: &mut BufStream<T>,
    ) -> Result<MessageBytes> {
        let mut message = MessageBytes::new();
        let mut remaining_bytes = buf_stream.read_u16().await? as usize;
        // Messages end in a 0_u16
        while remaining_bytes > 0 {
            let mut buf = vec![0; remaining_bytes];
            buf_stream.read_exact(&mut buf).await?;
            message.add_chunk(Chunk::try_from(Bytes::from(buf))?);
            remaining_bytes = buf_stream.read_u16().await? as usize;
        }
        Ok(message)
    }
}

impl Buf for MessageBytes {
    fn remaining(&self) -> usize {
        self.bytes.remaining()
    }

    fn bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

    fn advance(&mut self, cnt: usize) {
        self.bytes.advance(cnt)
    }
}

impl Serialize for MessageBytes {}

impl TryInto<Bytes> for MessageBytes {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u16>() + self.bytes.len() + mem::size_of::<u16>(),
        );
        bytes.put_u16(self.bytes.len() as u16);
        bytes.put(self.bytes);
        bytes.put_u16(0);
        Ok(bytes.freeze())
    }
}

impl Deserialize for MessageBytes {}

impl TryFrom<Arc<Mutex<Bytes>>> for MessageBytes {
    type Error = Error;

    fn try_from(value: Arc<Mutex<Bytes>>) -> Result<Self> {
        let bytes: &[u8] = &*value.lock().unwrap().clone();
        let bytes = BytesMut::from(bytes);
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::io::Cursor;

    use tokio::io::BufStream;

    use super::*;

    fn new_chunk() -> Chunk {
        Chunk::try_from(Bytes::from_static(&[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ]))
        .unwrap()
    }

    fn new_message() -> MessageBytes {
        let mut msg = MessageBytes::new();
        msg.add_chunk(new_chunk());
        msg
    }

    #[test]
    fn into_bytes() {
        let bytes: Bytes = new_message().try_into_bytes().unwrap();
        let mut result = BytesMut::new();
        result.put_u16(new_chunk().data.len() as u16);
        result.put(new_chunk().data);
        result.put_u16(0);
        assert_eq!(bytes, result.freeze())
    }

    #[tokio::test]
    async fn from_stream() {
        let bytes: Vec<u8> = vec![
            0x00, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x00,
        ];
        let mut stream = BufStream::new(Cursor::new(bytes));
        let message = MessageBytes::from_stream(&mut stream).await;
        assert_eq!(message.unwrap().bytes, new_chunk().data);
    }

    #[tokio::test]
    async fn from_stream_multiple_chunks() {
        let bytes: Vec<u8> = vec![
            0x00, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
            0x0C, 0x0D, 0x0E, 0x0F, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04, 0x00, 0x00,
        ];
        let mut stream = BufStream::new(Cursor::new(bytes));
        let message = MessageBytes::from_stream(&mut stream).await;
        assert_eq!(
            message.unwrap().bytes,
            Bytes::from_static(&[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F, 0x01, 0x02, 0x03, 0x04
            ])
        );
    }
}
