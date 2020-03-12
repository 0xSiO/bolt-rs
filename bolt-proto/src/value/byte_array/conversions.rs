use std::convert::TryFrom;

use crate::error::*;
use crate::value::ByteArray;
use crate::Value;

impl From<&[u8]> for ByteArray {
    fn from(value: &[u8]) -> Self {
        Self {
            value: Vec::from(value),
        }
    }
}

impl From<Vec<u8>> for ByteArray {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

impl From<ByteArray> for Vec<u8> {
    fn from(byte_array: ByteArray) -> Self {
        byte_array.value
    }
}

impl TryFrom<Value> for ByteArray {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Bytes(byte_array) => Ok(byte_array),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
