use std::convert::TryInto;

use failure::Error;

use crate::bolt::value::List;
use crate::error::ValueError;
use crate::Value;

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
            Value::List(list) => Ok(list.try_into()?),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}
