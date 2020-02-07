use std::convert::TryFrom;

use crate::error::*;
use crate::value::Integer;
use crate::Value;

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::value::Integer {
                fn from(value: $T) -> Self {
                    Self { bytes: ::bytes::BytesMut::from(value.to_be_bytes().as_ref()) }
                }
            }
        )*
    };
}
impl_from_int!(i8, i16, i32, i64);

macro_rules! impl_from_wrapped_int {
    ($($T:ty),+) => {
        $(
            impl From<crate::value::Integer> for $T {
                fn from(mut integer: crate::value::Integer) -> Self {
                    // Get bytes in little-endian order
                    integer.bytes.reverse();
                    integer.bytes.resize(::std::mem::size_of::<$T>(), 0);
                    let mut bytes: [u8; ::std::mem::size_of::<$T>()] = [0; ::std::mem::size_of::<$T>()];
                    bytes.copy_from_slice(&integer.bytes);
                    <$T>::from_le_bytes(bytes)
                }
            }
        )*
    };
}
impl_from_wrapped_int!(i8, i16, i32, i64);

impl TryFrom<Value> for Integer {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Integer(integer) => Ok(integer),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

macro_rules! impl_from_value {
    ($($T:ty),+) => {
        $(
            impl TryFrom<crate::Value> for $T {
                type Error = crate::error::Error;

                fn try_from(value: crate::Value) -> crate::error::Result<Self> {
                    match value {
                        crate::Value::Integer(integer) => Ok(<$T>::from(integer)),
                        _ => Err(crate::error::ValueError::InvalidConversion(value).into()),
                    }
                }
            }
        )*
    };
}
impl_from_value!(i8, i16, i32, i64);
