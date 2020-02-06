use std::convert::TryFrom;

use crate::v1::error::*;
use crate::v1::value::Integer;
use crate::v1::Value;

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::v1::value::Integer {
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
            impl From<crate::v1::value::Integer> for $T {
                fn from(mut integer: crate::v1::value::Integer) -> Self {
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
            impl TryFrom<crate::v1::Value> for $T {
                type Error = crate::v1::error::Error;

                fn try_from(value: crate::v1::Value) -> crate::v1::error::Result<Self> {
                    match value {
                        crate::v1::Value::Integer(integer) => Ok(<$T>::from(integer)),
                        _ => Err(crate::v1::error::ValueError::InvalidConversion(value).into()),
                    }
                }
            }
        )*
    };
}
impl_from_value!(i8, i16, i32, i64);
