use std::convert::TryFrom;

use crate::error::*;
use crate::value::Boolean;
use crate::Value;

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(boolean) => Ok(boolean.value),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
