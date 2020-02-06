use std::convert::TryFrom;

use crate::bolt::value::List;
use crate::error::*;
use crate::Value;

impl<T> From<Vec<T>> for List
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        Self {
            value: value.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl TryFrom<Value> for List {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => Ok(list),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
