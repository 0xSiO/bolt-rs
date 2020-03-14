use std::convert::TryFrom;

use crate::impl_try_from_value;
use crate::value::Integer;

macro_rules! impl_from_primitives_for_integer {
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
impl_from_primitives_for_integer!(i8, i16, i32, i64);

macro_rules! impl_from_integer_for_primitives {
    ($($T:ty),+) => {
        $(
            impl From<crate::value::Integer> for $T {
                fn from(mut integer: crate::value::Integer) -> Self {
                    // If positive, extend with zeros. If negative, extend with ones.
                    let extension = if integer.bytes[0] & 0b10000000 == 0 {
                                        0x00
                                    } else {
                                        0xFF
                                    };
                    // Get bytes in little-endian order
                    integer.bytes.reverse();
                    integer.bytes.resize(::std::mem::size_of::<$T>(), extension);
                    let mut bytes: [u8; ::std::mem::size_of::<$T>()] = [0; ::std::mem::size_of::<$T>()];
                    bytes.copy_from_slice(&integer.bytes);
                    <$T>::from_le_bytes(bytes)
                }
            }
        )*
    };
}
impl_from_integer_for_primitives!(i8, i16, i32, i64);

impl_try_from_value!(Integer, Integer);

macro_rules! impl_try_from_value_for_primitives {
    ($($T:ty),+) => {
        $(
            impl TryFrom<crate::Value> for $T {
                type Error = crate::error::Error;

                fn try_from(value: crate::Value) -> crate::error::Result<Self> {
                    match value {
                        crate::Value::Integer(integer) => Ok(<$T>::from(integer)),
                        _ => Err(crate::error::ConversionError::FromValue(value).into()),
                    }
                }
            }
        )*
    };
}
impl_try_from_value_for_primitives!(i8, i16, i32, i64);
