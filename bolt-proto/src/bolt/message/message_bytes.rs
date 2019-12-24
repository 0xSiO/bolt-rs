use std::convert::TryFrom;
use std::mem;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use failure::Error;
use tokio::io::BufStream;
use tokio::prelude::*;

use crate::bolt::message::Chunk;

#[derive(Debug)]
pub struct MessageBytes {
    bytes: BytesMut,
}

impl MessageBytes {
    pub fn new() -> MessageBytes {
        MessageBytes {
            bytes: BytesMut::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn split_to(&mut self, at: usize) -> Bytes {
        self.bytes.split_to(at).freeze()
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.bytes.put(chunk.data);
    }

    pub async fn from_stream<T: Unpin + AsyncRead + AsyncWrite>(
        buf_stream: &mut BufStream<T>,
    ) -> Result<MessageBytes, Error> {
        let mut message = MessageBytes::new();
        loop {
            let size = buf_stream.read_u16().await? as usize;
            if size == 0 {
                // We've reached the end of the message
                // Note that after this point we will have consumed the last two 0 bytes
                break;
            }
            let mut buf = BytesMut::with_capacity(size);
            buf_stream.read_buf(&mut buf).await?;
            debug_assert!(buf.len() == size);
            message.add_chunk(Chunk::try_from(buf.freeze())?)
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

impl Into<Bytes> for MessageBytes {
    // TODO: This puts the message into a single chunk, consider breaking up large messages into several chunk
    fn into(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() * 2 + self.len() + mem::size_of::<u8>() * 2,
        );
        bytes.put_u16(self.len() as u16);
        bytes.put(self.bytes);
        bytes.put_u16(0);
        bytes.freeze()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use tokio::io::BufStream;

    use super::*;
    use std::io::Cursor;

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
        let bytes: Bytes = new_message().into();
        let mut result = BytesMut::new();
        result.put_u16(new_chunk().data.len() as u16);
        result.put(new_chunk().data);
        result.put_u16(0);
        assert_eq!(bytes, result.freeze())
    }

    //    #[test]
    //    fn into_bytes_multiple_chunks() {
    //        todo!();
    //    }

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
