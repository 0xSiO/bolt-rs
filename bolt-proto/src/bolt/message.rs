use std::convert::TryFrom;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use failure::Error;

pub use chunk::Chunk;
pub use init::Init;
pub use message_bytes::MessageBytes;
pub use success::Success;

use crate::bolt::structure::*;
use crate::error::DeserializeError;

mod chunk;
mod init;
mod message_bytes;
mod success;

#[derive(Debug)]
pub enum Message {
    Init(Init),
    Success(Success),
}

impl TryFrom<MessageBytes> for Message {
    type Error = Error;

    fn try_from(mut message_bytes: MessageBytes) -> Result<Self, Self::Error> {
        let result: Result<Message, Error> = catch_unwind(move || {
            let signature = get_signature_from_bytes(&mut message_bytes)?;
            let remaining_bytes_arc =
                Arc::new(Mutex::new(message_bytes.split_to(message_bytes.len())));

            match signature {
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
