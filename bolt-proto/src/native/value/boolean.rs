use std::convert::TryFrom;

use failure::Error;

use crate::bolt::value::{Boolean, Value};
use crate::error::ValueError;

// TODO: This seems a little silly, consider removing the Boolean type if possible
impl From<Boolean> for bool {
    fn from(boolean: Boolean) -> Self {
        boolean.value
    }
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(boolean) => Ok(bool::from(boolean)),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
