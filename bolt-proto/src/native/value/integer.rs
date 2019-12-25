use std::convert::TryFrom;

macro_rules! impl_from_integer {
    ($($T:ty),+) => {
        $(
            impl From<crate::bolt::value::Integer> for $T {
                fn from(mut integer: crate::bolt::value::Integer) -> Self {
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
impl_from_integer!(i8, i16, i32, i64);

macro_rules! impl_from_value {
    ($($T:ty),+) => {
        $(
            impl TryFrom<crate::Value> for $T {
                type Error = ::failure::Error;

                fn try_from(value: crate::Value) -> Result<Self, Self::Error> {
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
