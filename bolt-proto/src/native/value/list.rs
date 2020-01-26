use std::convert::{TryFrom, TryInto};

use crate::bolt::value::List;
use crate::error::Error;
use crate::error::ValueError;
use crate::Value;

impl<T> TryInto<Vec<T>> for List
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        self.value
            .into_iter()
            .map(|value| T::try_from(value))
            .collect()
    }
}

impl<T> TryInto<Vec<T>> for Value
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        match self {
            Value::List(list) => list.try_into(),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}

impl TryInto<Vec<Value>> for List {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Value>, Self::Error> {
        Ok(self.value)
    }
}

impl TryInto<Vec<Value>> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Value>, Self::Error> {
        match self {
            Value::List(list) => list.try_into(),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}
