use std::convert::TryFrom;

use failure::Error;

use crate::bolt::value::{Integer, Value};
use crate::error::ValueError;

impl From<Integer> for i64 {
    fn from(mut integer: Integer) -> Self {
        // Get bytes in little-endian order
        integer.bytes.reverse();
        integer.bytes.resize(8, 0);
        let mut bytes: [u8; 8] = [0; 8];
        bytes.copy_from_slice(&integer.bytes);
        i64::from_le_bytes(bytes)
    }
}

impl TryFrom<Value> for i64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Integer(integer) => Ok(i64::from(integer)),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
