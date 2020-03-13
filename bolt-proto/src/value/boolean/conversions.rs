use std::convert::TryFrom;

use crate::error::*;
use crate::value::Boolean;
use crate::Value;

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl From<Boolean> for bool {
    fn from(boolean: Boolean) -> Self {
        boolean.value
    }
}

impl TryFrom<Value> for Boolean {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        Ok(Boolean::from(bool::try_from(value)?))
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(boolean) => Ok(boolean),
            _ => Err(Error::InvalidValueConversion(value).into()),
        }
    }
}
