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

// We don't need TryFrom<Value> for List since it can be converted directly into a Vec
// impl_try_from_value!(List, List);

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => list.try_into(),
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
