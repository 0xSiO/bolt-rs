use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use bytes::Bytes;
use failure::Error;
use failure::_core::convert::TryInto;

use crate::bolt::value::{BoltValue, Marker};
use crate::error::ValueError;
use crate::serialize::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct Float {
    value: f64,
}

impl Hash for Float {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!("Cannot hash a Float")
    }
}

impl Eq for Float {
    fn assert_receiver_is_total_eq(&self) {
        panic!("Floats cannot be Eq")
    }
}

impl TryFrom<BoltValue> for Float {
    type Error = Error;

    fn try_from(value: BoltValue) -> Result<Self, Self::Error> {
        match value {
            BoltValue::Float(float) => Ok(float),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

impl Marker for Float {
    fn get_marker(&self) -> Result<u8, Error> {
        todo!()
    }
}

impl Serialize for Float {}

impl TryInto<Bytes> for Float {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        todo!()
    }
}

impl Deserialize for Float {}

impl TryFrom<Arc<Mutex<Bytes>>> for Float {
    type Error = Error;

    fn try_from(_value: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        todo!()
    }
}
