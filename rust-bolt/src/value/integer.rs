use std::convert::{TryFrom, TryInto};
use std::mem;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use failure::Error;

use crate::error::{DeserializeError, SerializeError, ValueError};
use crate::serialize::{Deserialize, Serialize};
use crate::value::{Marker, Value};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

const MARKER_INT_8: u8 = 0xC8;
const MARKER_INT_16: u8 = 0xC9;
const MARKER_INT_32: u8 = 0xCA;
const MARKER_INT_64: u8 = 0xCB;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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

            impl From<$T> for $crate::value::Value {
                fn from(value: $T) -> Self {
                    Value::Integer(value.into())
                }
            }
        )*
    };
}

impl_from_int!(i8, i16, i32, i64);

impl Marker for Integer {
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

impl Serialize for Integer {}

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

impl Deserialize for Integer {}

impl TryFrom<Arc<Mutex<Bytes>>> for Integer {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        let result: Result<Integer, Error> = catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
            let marker = input_bytes.get_u8();
            if !input_bytes.has_remaining() {
                // Tiny int
                return Ok(Integer::from(marker as i8));
            }

            match marker {
                MARKER_INT_8 => Ok(Integer::from(input_bytes.get_i8())),
                MARKER_INT_16 => Ok(Integer::from(input_bytes.get_i16())),
                MARKER_INT_32 => Ok(Integer::from(input_bytes.get_i32())),
                MARKER_INT_64 => Ok(Integer::from(input_bytes.get_i64())),
                _ => Err(DeserializeError(format!("Invalid marker byte: {:x}", marker)).into()),
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Integer from Bytes: {}", err))
        })?)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::value::Marker;

    use super::*;

    #[test]
    fn get_marker() {
        let tiny = Integer::from(-16_i8);
        assert_eq!(tiny.get_marker().unwrap(), 0xF0);
        let small = Integer::from(-50_i8);
        assert_eq!(small.get_marker().unwrap(), MARKER_INT_8);
        let medium = Integer::from(-8000_i16);
        assert_eq!(medium.get_marker().unwrap(), MARKER_INT_16);
        let large = Integer::from(-1_000_000_000_i32);
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
        let medium = Integer::from(-8000_i16);
        assert_eq!(
            medium.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_INT_16, 0xE0, 0xC0])
        );
        let large = Integer::from(-1_000_000_000_i32);
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

    #[test]
    fn try_from_bytes() {
        let tiny = Integer::from(-16_i8);
        assert_eq!(
            Integer::try_from(Arc::new(Mutex::new(tiny.clone().try_into_bytes().unwrap())))
                .unwrap(),
            tiny
        );
        let small = Integer::from(-50_i8);
        assert_eq!(
            Integer::try_from(Arc::new(Mutex::new(
                small.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            small
        );
        let medium = Integer::from(-8000_i16);
        assert_eq!(
            Integer::try_from(Arc::new(Mutex::new(
                medium.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            medium
        );
        let large = Integer::from(-1_000_000_000_i32);
        assert_eq!(
            Integer::try_from(Arc::new(Mutex::new(
                large.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            large
        );
        let very_large = Integer::from(-9_000_000_000_000_000_000_i64);
        assert_eq!(
            Integer::try_from(Arc::new(Mutex::new(
                very_large.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            very_large
        );
    }
}
