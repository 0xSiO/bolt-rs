use std::convert::TryFrom;

use crate::error::*;
use crate::impl_try_from_value;
use crate::value::String;
use crate::Value;

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self { value }
    }
}

impl From<String> for std::string::String {
    fn from(string: String) -> Self {
        string.value
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::String(string) => Ok(String::from(string)),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

// TODO: Consider implementing this for the wrapper type rather than the std type
// Converting to the wrapper type involves copying the std type at the moment
impl_try_from_value!(std::string::String, String);
