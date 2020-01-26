use std::convert::TryFrom;

use crate::bolt::value::Float;
use crate::error::*;
use crate::Value;

impl From<Float> for f64 {
    fn from(float: Float) -> Self {
        float.value
    }
}

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Float(float) => Ok(float),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
