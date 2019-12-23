use std::convert::{TryFrom, TryInto};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use failure::Error;

use crate::bolt::value::{BoltValue, Marker};
use crate::error::ValueError;
use crate::serialize::{Deserialize, Serialize};

pub const MARKER_TINY: u8 = 0x90;
pub const MARKER_SMALL: u8 = 0xD4;
pub const MARKER_MEDIUM: u8 = 0xD5;
pub const MARKER_LARGE: u8 = 0xD6;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct List {
    pub(crate) value: Vec<BoltValue>,
}

impl<T> From<Vec<T>> for List
where
    T: Into<BoltValue>,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            value: value.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl TryFrom<BoltValue> for List {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::List(list) => Ok(list),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

impl<T> From<Vec<T>> for BoltValue
where
    T: Into<BoltValue>,
{
    fn from(value: Vec<T>) -> Self {
        BoltValue::List(value.into())
    }
}

impl Marker for List {
    fn get_marker(&self) -> Result<u8, Error> {
        todo!()
    }
}

impl Serialize for List {}

impl TryInto<Bytes> for List {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        todo!()
    }
}

impl Deserialize for List {}

impl TryFrom<Arc<Mutex<Bytes>>> for List {
    type Error = Error;

    fn try_from(_value: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_marker() {
        todo!()
    }

    #[test]
    fn try_into_bytes() {
        todo!()
    }

    #[test]
    fn try_from_bytes() {
        todo!()
    }
}
