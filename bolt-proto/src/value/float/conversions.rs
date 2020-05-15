use std::convert::TryFrom;

use crate::error::*;
use crate::value::Float;
use crate::Value;

impl From<f64> for Float {
    fn from(float: f64) -> Self {
        Self { value: float }
    }
}

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Float(float) => Ok(float.value),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
