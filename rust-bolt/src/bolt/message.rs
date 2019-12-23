use std::convert::TryFrom;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::Buf;
use failure::Error;

pub use chunk::Chunk;
pub use init::BoltInit;
pub use message_bytes::BoltMessageBytes;
pub use success::BoltSuccess;

use crate::error::DeserializeError;
use crate::structure::*;

mod chunk;
mod init;
mod message_bytes;
mod success;

#[derive(Debug)]
pub enum BoltMessage {
    Init(BoltInit),
    Success(BoltSuccess),
}

impl TryFrom<BoltMessageBytes> for BoltMessage {
    type Error = Error;

    fn try_from(mut message_bytes: BoltMessageBytes) -> Result<Self, Self::Error> {
        let result: Result<BoltMessage, Error> = catch_unwind(move || {
            let marker = message_bytes.get_u8();
            let _size = match marker {
                marker if (MARKER_TINY..=(MARKER_TINY | 0x0F)).contains(&marker) => {
                    0x0F & marker as usize
                }
                MARKER_SMALL => message_bytes.get_u8() as usize,
                MARKER_MEDIUM => message_bytes.get_u16() as usize,
                _ => {
                    return Err(
                        DeserializeError(format!("Invalid marker byte: {:x}", marker)).into(),
                    );
                }
            };
            let signature = message_bytes.get_u8();
            let remaining_bytes_arc =
                Arc::new(Mutex::new(message_bytes.split_to(message_bytes.len())));

            match signature {
                success::SIGNATURE => Ok(BoltMessage::Success(BoltSuccess::try_from(
                    remaining_bytes_arc,
                )?)),
                _ => {
                    Err(DeserializeError(format!("Invalid signature byte: {:x}", signature)).into())
                }
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating BoltMessage from Bytes: {}", err))
        })?)
    }
}
