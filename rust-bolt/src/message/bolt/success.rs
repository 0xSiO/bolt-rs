use std::convert::TryFrom;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, Bytes};
use failure::Error;

use rust_bolt_macros::*;

use crate::error::DeserializeError;
use crate::serialize::{Deserialize, Serialize};
use crate::structure::Structure;
use crate::value::Value;

#[derive(Debug, Structure)]
pub struct BoltSuccess {
    metadata: Value,
}

// TODO: You may be able to move this all into a derive macro
impl Deserialize for BoltSuccess {}

const MARKER_TINY_STRUCTURE: u8 = 0xB0;
const MARKER_SMALL_STRUCTURE: u8 = 0xDC;
const MARKER_MEDIUM_STRUCTURE: u8 = 0xDD;

impl TryFrom<Arc<Mutex<Bytes>>> for BoltSuccess {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        let result: Result<BoltSuccess, Error> = catch_unwind(move || {
            let marker = input_arc.lock().unwrap().get_u8();
            let size = match marker {
                marker
                    if (MARKER_TINY_STRUCTURE..=(MARKER_TINY_STRUCTURE | 0x0F))
                        .contains(&marker) =>
                {
                    0x0F & marker as usize
                }
                MARKER_SMALL_STRUCTURE => input_arc.lock().unwrap().get_u8() as usize,
                MARKER_MEDIUM_STRUCTURE => input_arc.lock().unwrap().get_u16() as usize,
                _ => {
                    return Err(
                        DeserializeError(format!("Invalid marker byte: {:x}", marker)).into(),
                    );
                }
            };
            let signature = input_arc.lock().unwrap().get_u8();

            if signature == 0x70 {
                Ok(BoltSuccess {
                    metadata: Value::try_from(Arc::clone(&input_arc))?,
                })
            } else {
                Err(DeserializeError(format!("Invalid signature byte: {:x}", signature)).into())
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Success from Bytes: {}", err))
        })?)
    }
}

#[cfg(test)]
mod tests {
    //    #[test]
    //    fn try_from_bytes() {
    //        todo!()
    //    }
}
