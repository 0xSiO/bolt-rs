use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use failure::Error;

use rust_bolt_macros::*;

use crate::bolt::value::BoltValue;
use crate::serialize::{Deserialize, Serialize};
use crate::structure::*;

pub const SIGNATURE: u8 = 0x70;

#[derive(Debug, Structure)]
pub struct BoltSuccess {
    metadata: BoltValue,
}

// TODO: You may be able to move this all into a derive macro
impl Deserialize for BoltSuccess {}

impl TryFrom<Arc<Mutex<Bytes>>> for BoltSuccess {
    type Error = Error;

    fn try_from(remaining_bytes_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        Ok(BoltSuccess {
            metadata: BoltValue::try_from(Arc::clone(&remaining_bytes_arc))?,
        })
    }
}

#[cfg(test)]
mod tests {
    //    #[test]
    //    fn try_from_bytes() {
    //        todo!()
    //    }
}
