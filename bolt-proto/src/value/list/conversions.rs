use std::convert::TryFrom;

use crate::error::*;
use crate::value::List;
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

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => list.value.into_iter().map(T::try_from).collect(),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => Ok(list.value),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
