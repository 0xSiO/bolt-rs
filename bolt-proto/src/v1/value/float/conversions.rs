use std::convert::TryFrom;

use crate::v1::error::*;
use crate::v1::value::Float;
use crate::v1::Value;

impl From<f64> for Float {
    fn from(float: f64) -> Self {
        Self { value: float }
    }
}

impl From<Float> for f64 {
    fn from(float: Float) -> Self {
        float.value
    }
}

impl TryFrom<Value> for Float {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        Ok(Float::from(f64::try_from(value)?))
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
