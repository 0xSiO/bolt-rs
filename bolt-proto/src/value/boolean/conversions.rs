use std::convert::TryFrom;

use crate::error::*;
use crate::impl_try_from_value;
use crate::value::Boolean;
use crate::Value;

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl From<Boolean> for bool {
    fn from(boolean: Boolean) -> Self {
        boolean.value
    }
}

impl TryFrom<Value> for Boolean {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        Ok(Boolean::from(bool::try_from(value)?))
    }
}

impl_try_from_value!(bool, Boolean);
