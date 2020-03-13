use std::convert::TryFrom;

use crate::error::*;
use crate::impl_try_from_value;
use crate::value::Float;
use crate::Value;

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

impl_try_from_value!(f64, Float);
