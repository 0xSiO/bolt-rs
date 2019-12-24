use std::convert::{TryFrom, TryInto};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use failure::Error;
use tokio::io::BufStream;
use tokio::prelude::*;

pub(crate) use chunk::Chunk;
pub use init::Init;
pub(crate) use message_bytes::MessageBytes;
pub use success::Success;

use crate::bolt::structure::get_signature_from_bytes;
use crate::error::DeserializeError;
use crate::native;

mod chunk;
mod init;
mod message_bytes;
mod success;

// This is what's used in the protocol spec, but it could technically be any size.
const CHUNK_SIZE: usize = 16;

#[derive(Debug)]
pub enum Message {
    Init(Init),
    Success(Success),
}

impl Message {
    pub async fn from_stream<T: Unpin + AsyncRead + AsyncWrite>(
        buf_stream: &mut BufStream<T>,
    ) -> Result<Message, Error> {
        Message::try_from(MessageBytes::from_stream(buf_stream).await?)
    }
}

impl From<native::message::Init> for Message {
    fn from(message: native::message::Init) -> Self {
        Message::Init(Init::from(message))
    }
}

impl From<native::message::Success> for Message {
    fn from(message: native::message::Success) -> Self {
        Message::Success(Success::from(message))
    }
}

impl TryFrom<MessageBytes> for Message {
    type Error = Error;

    fn try_from(mut message_bytes: MessageBytes) -> Result<Self, Self::Error> {
        let result: Result<Message, Error> = catch_unwind(move || {
            let signature = get_signature_from_bytes(&mut message_bytes)?;
            let remaining_bytes_arc =
                Arc::new(Mutex::new(message_bytes.split_to(message_bytes.len())));

            match signature {
                init::SIGNATURE => Ok(Message::Init(Init::try_from(remaining_bytes_arc)?)),
                success::SIGNATURE => Ok(Message::Success(Success::try_from(remaining_bytes_arc)?)),
                _ => {
                    Err(DeserializeError(format!("Invalid signature byte: {:x}", signature)).into())
                }
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Message from Bytes: {}", err))
        })?)
    }
}

impl TryInto<Vec<Bytes>> for Message {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Bytes>, Self::Error> {
        let bytes: Bytes = match self {
            Message::Init(init) => init.try_into()?,
            Message::Success(success) => success.try_into()?,
        };

        // Big enough to hold all the chunks, plus a partial chunk, plus the message footer
        let mut result: Vec<Bytes> = Vec::with_capacity(bytes.len() / CHUNK_SIZE + 2);
        for slice in bytes.chunks(CHUNK_SIZE) {
            let chunk_bytes: Bytes = Chunk::try_from(Bytes::copy_from_slice(slice))?.into();
            result.push(chunk_bytes);
        }
        // End message
        result.push(Bytes::from_static(&[0, 0]));

        Ok(result)
    }
}
