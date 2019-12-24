use std::convert::{TryFrom, TryInto};

use failure::Error;

use crate::bolt::value::List;
use crate::error::ValueError;
use crate::Value;

impl<T> TryInto<Vec<T>> for List
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        let mut vec: Vec<T> = Vec::with_capacity(self.value.len());
        for value in self.value {
            vec.push(T::try_from(value)?);
        }
        Ok(vec)
    }
}

impl<T> TryInto<Vec<T>> for Value
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<Vec<T>, Self::Error> {
        match self {
            Value::List(list) => Ok(list.try_into()?),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}
