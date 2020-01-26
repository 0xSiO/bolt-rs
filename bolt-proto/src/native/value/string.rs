use std::convert::TryFrom;

use crate::bolt;
use crate::bolt::Value;
use crate::error::*;

impl From<bolt::value::String> for String {
    fn from(string: bolt::value::String) -> Self {
        string.value
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::String(string) => Ok(string),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
