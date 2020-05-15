use std::convert::TryFrom;

macro_rules! impl_from_primitives_for_integer {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::value::Integer {
                fn from(value: $T) -> Self {
                    Self { value: value as i64 }
                }
            }
        )*
    };
}
impl_from_primitives_for_integer!(i8, i16, i32, i64);

macro_rules! impl_try_from_value_for_primitives {
    ($($T:ty),+) => {
        $(
            impl TryFrom<crate::Value> for $T {
                type Error = crate::error::Error;

                fn try_from(value: crate::Value) -> crate::error::Result<Self> {
                    match value {
                        crate::Value::Integer(integer) => Ok(integer.value as $T),
                        _ => Err(crate::error::ConversionError::FromValue(value).into()),
                    }
                }
            }
        )*
    };
}
impl_try_from_value_for_primitives!(i8, i16, i32, i64);
