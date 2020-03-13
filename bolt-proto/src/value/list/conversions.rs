use std::convert::{TryFrom, TryInto};

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

impl TryFrom<Value> for List {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => Ok(list),
            _ => Err(Error::InvalidValueConversion(value).into()),
        }
    }
}

impl<T> TryInto<Vec<T>> for List
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<Vec<T>> {
        self.value.into_iter().map(T::try_from).collect()
    }
}

impl TryInto<Vec<Value>> for List {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Value>> {
        Ok(self.value)
    }
}
