use crate::impl_try_from_value;
use crate::value::Boolean;

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        Self { value }
    }
}

impl_try_from_value!(bool, Boolean);
