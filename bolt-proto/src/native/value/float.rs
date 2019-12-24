use std::convert::TryFrom;

use failure::Error;

use crate::bolt::value::{Float, Value};
use crate::error::ValueError;

// TODO: Seems a little silly, consider removing Float type if possible
impl From<Float> for f64 {
    fn from(float: Float) -> Self {
        float.value
    }
}

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Float(float) => Ok(f64::from(float)),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
