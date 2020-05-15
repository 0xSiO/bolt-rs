use std::convert::TryFrom;

use crate::error::*;
use crate::value::ByteArray;
use crate::Value;

impl From<Vec<u8>> for ByteArray {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Bytes(byte_array) => Ok(byte_array.value),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
