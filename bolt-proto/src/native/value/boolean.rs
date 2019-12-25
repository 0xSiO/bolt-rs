use std::convert::TryFrom;

use failure::Error;

use crate::bolt::value::Boolean;
use crate::error::ValueError;
use crate::Value;

impl From<Boolean> for bool {
    fn from(boolean: Boolean) -> Self {
        boolean.value
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(boolean) => Ok(boolean),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
