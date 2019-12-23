use std::convert::TryFrom;
use std::panic::catch_unwind;

use bytes::{Buf, Bytes};
use failure::Error;

pub use bolt::init::BoltInit;
pub use bolt::success::BoltSuccess;
pub use chunk::Chunk;
pub use message_bytes::BoltMessageBytes;

use crate::error::DeserializeError;
use crate::structure::*;

mod bolt;
mod chunk;
mod message_bytes;

#[derive(Debug)]
pub enum BoltMessage {
    Init(BoltInit),
    Success(BoltSuccess),
}

impl TryFrom<BoltMessageBytes> for BoltMessage {
    type Error = Error;

    fn try_from(message_bytes: BoltMessageBytes) -> Result<Self, Self::Error> {
        let result: Result<BoltMessage, Error> = catch_unwind(move || {
            let mut bytes: Bytes = message_bytes.into();
            let mut temp_bytes = bytes.clone();
            let marker = temp_bytes.get_u8();
            let size = match marker {
                marker
                    if (MARKER_TINY_STRUCTURE..=(MARKER_TINY_STRUCTURE | 0x0F))
                        .contains(&marker) =>
                {
                    0x0F & marker as usize
                }
                MARKER_SMALL_STRUCTURE => temp_bytes.get_u8() as usize,
                MARKER_MEDIUM_STRUCTURE => temp_bytes.get_u16() as usize,
                _ => {
                    return Err(
                        DeserializeError(format!("Invalid marker byte: {:x}", marker)).into(),
                    );
                }
            };
            let signature = temp_bytes.get_u8();

            //            match signature {}
            todo!()
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating BoltMessage from Bytes: {}", err))
        })?)
    }
}
