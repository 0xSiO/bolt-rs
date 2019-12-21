use std::convert::TryInto;
use std::mem;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use failure::Error;

use crate::error::{SerializeError, ValueError};
use crate::value::Value;

const MARKER_INT_8: u8 = 0xC8;
const MARKER_INT_16: u8 = 0xC9;
const MARKER_INT_32: u8 = 0xCA;
const MARKER_INT_64: u8 = 0xCB;

#[derive(Debug)]
pub struct Integer {
    // Since integers come in many sizes, just store the bytes directly
    bytes: Bytes,
}

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::value::Integer {
                fn from(value: $T) -> Self {
                    Self { bytes: ::bytes::Bytes::copy_from_slice(&value.to_be_bytes()) }
                }
            }
        )*
    };
}

impl_from_int!(i8, i16, i32, i64);

impl Value for Integer {
    fn get_marker(&self) -> Result<u8, Error> {
        let value = match self.bytes.len() {
            1 => self.bytes.clone().get_i8() as i64,
            2 => self.bytes.clone().get_i16() as i64,
            4 => self.bytes.clone().get_i32() as i64,
            8 => self.bytes.clone().get_i64() as i64,
            _ => return Err(ValueError::TooLarge(self.bytes.len()).into()),
        };
        match value {
            -9_223_372_036_854_775_808..=-2_147_483_649
            | 2_147_483_648..=9_223_372_036_854_775_807 => Ok(MARKER_INT_64),
            -2_147_483_648..=-32_769 | 32_768..=2_147_483_647 => Ok(MARKER_INT_32),
            -32_768..=-129 | 128..=32_767 => Ok(MARKER_INT_16),
            -128..=-17 => Ok(MARKER_INT_8),
            -16..=127 => Ok((value as i8).to_be_bytes()[0]),
        }
    }
}

impl TryInto<Bytes> for Integer {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        let mut bytes = BytesMut::with_capacity(mem::size_of::<u8>() + self.bytes.len());
        bytes.put_u8(self.get_marker()?);
        let first_byte = *self.bytes.get(0).ok_or(SerializeError(format!(
            "Unable to get first element of bytes: {:?}",
            self.bytes
        )))?;
        // Anything other than tiny integers need the rest of their bytes added
        if self.get_marker()? != first_byte {
            bytes.put(self.bytes);
        }
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::value::Value;

    use super::*;

    #[test]
    fn get_marker() {
        let tiny = Integer::from(-16);
        assert_eq!(tiny.get_marker().unwrap(), 0xF0);
        let small = Integer::from(-50);
        assert_eq!(small.get_marker().unwrap(), MARKER_INT_8);
        let medium = Integer::from(-8000);
        assert_eq!(medium.get_marker().unwrap(), MARKER_INT_16);
        let large = Integer::from(-1_000_000_000);
        assert_eq!(large.get_marker().unwrap(), MARKER_INT_32);
        let very_large = Integer::from(-9_000_000_000_000_000_000_i64);
        assert_eq!(very_large.get_marker().unwrap(), MARKER_INT_64);
    }

    #[test]
    fn try_into_bytes() {
        let tiny = Integer::from(-16_i8);
        assert_eq!(tiny.try_into_bytes().unwrap(), Bytes::from_static(&[0xF0]));
        let small = Integer::from(-50_i8);
        assert_eq!(
            small.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_INT_8, 0xCE])
        );
        let medium = Integer::from(-8000);
        assert_eq!(
            medium.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_INT_16, 0xFF, 0xFF, 0xE0, 0xC0])
        );
        let large = Integer::from(-1_000_000_000);
        assert_eq!(
            large.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_INT_32, 0xC4, 0x65, 0x36, 0x00])
        );
        let very_large = Integer::from(-9_000_000_000_000_000_000_i64);
        assert_eq!(
            very_large.try_into_bytes().unwrap(),
            Bytes::from_static(&[
                MARKER_INT_64,
                0x83,
                0x19,
                0x93,
                0xAF,
                0x1D,
                0x7C,
                0x00,
                0x00
            ])
        );
    }
}
