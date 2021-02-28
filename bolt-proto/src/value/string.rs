use std::convert::{TryFrom, TryInto};
use std::mem;
use std::panic::catch_unwind;
use std::str;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::*;
use crate::serialization::*;

pub(crate) const MARKER_TINY: u8 = 0x80;
pub(crate) const MARKER_SMALL: u8 = 0xD0;
pub(crate) const MARKER_MEDIUM: u8 = 0xD1;
pub(crate) const MARKER_LARGE: u8 = 0xD2;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct String {
    pub(crate) value: std::string::String,
}

impl Marker for String {
    fn get_marker(&self) -> Result<u8> {
        match self.value.len() {
            0..=15 => Ok(MARKER_TINY | self.value.len() as u8),
            16..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=4_294_967_295 => Ok(MARKER_LARGE),
            _ => Err(Error::ValueTooLarge(self.value.len())),
        }
    }
}

impl Serialize for String {}

impl TryInto<Bytes> for String {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        let marker = self.get_marker()?;
        let length = self.value.len();
        // Worst case is a large string, with marker byte, 32-bit size value, and length
        let mut bytes =
            BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<u32>() + length);
        bytes.put_u8(marker);
        match length {
            0..=15 => {}
            16..=255 => bytes.put_u8(length as u8),
            256..=65_535 => bytes.put_u16(length as u16),
            65_536..=4_294_967_295 => bytes.put_u32(length as u32),
            _ => return Err(Error::ValueTooLarge(length)),
        }
        bytes.put_slice(self.value.as_bytes());
        Ok(bytes.freeze())
    }
}

impl Deserialize for String {}

impl TryFrom<Arc<Mutex<Bytes>>> for String {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self> {
        catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
            let marker = input_bytes.get_u8();
            let size = match marker {
                // Lower-order nibble of tiny string marker
                0x80..=0x8F => 0x0F & marker as usize,
                MARKER_SMALL => input_bytes.get_u8() as usize,
                MARKER_MEDIUM => input_bytes.get_u16() as usize,
                MARKER_LARGE => input_bytes.get_u32() as usize,
                _ => {
                    return Err(DeserializationError::InvalidMarkerByte(marker).into());
                }
            };
            let mut string_bytes = BytesMut::with_capacity(size);
            // We resize here so that the length of string_bytes is nonzero, which allows
            // us to use copy_to_slice
            string_bytes.resize(size, 0);
            input_bytes.copy_to_slice(&mut string_bytes);
            Ok(String::from(
                str::from_utf8(&string_bytes).map_err(DeserializationError::InvalidUTF8)?,
            ))
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

impl From<&str> for String {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self { value }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use super::*;

    #[test]
    fn get_marker() {
        let tiny = String::from("string");
        assert_eq!(
            tiny.get_marker().unwrap(),
            MARKER_TINY | tiny.value.len() as u8
        );
        let small = String::from("string".repeat(10));
        assert_eq!(small.get_marker().unwrap(), MARKER_SMALL);
        let medium = String::from("string".repeat(1000));
        assert_eq!(medium.get_marker().unwrap(), MARKER_MEDIUM);
        let large = String::from("string".repeat(100_000));
        assert_eq!(large.get_marker().unwrap(), MARKER_LARGE);
    }

    #[test]
    fn try_into_bytes() {
        let tiny_bytes = String::from("a").try_into_bytes().unwrap();
        assert_eq!(tiny_bytes, Bytes::from_static(&[0x81, 0x61]));
        let normal_bytes = String::from("abcdefghijklmnopqrstuvwxyz")
            .try_into_bytes()
            .unwrap();
        assert_eq!(
            normal_bytes,
            Bytes::from_static(&[
                0xD0, 0x1A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C,
                0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A
            ])
        );
        let special_bytes = String::from("En å flöt över ängen")
            .try_into_bytes()
            .unwrap();
        assert_eq!(
            special_bytes,
            Bytes::from_static(&[
                0xD0, 0x18, 0x45, 0x6E, 0x20, 0xC3, 0xA5, 0x20, 0x66, 0x6C, 0xC3, 0xB6, 0x74, 0x20,
                0xC3, 0xB6, 0x76, 0x65, 0x72, 0x20, 0xC3, 0xA4, 0x6E, 0x67, 0x65, 0x6E
            ])
        );
    }

    #[test]
    fn try_from_bytes() {
        let tiny = String::from("string");
        assert_eq!(
            String::try_from(Arc::new(Mutex::new(tiny.clone().try_into_bytes().unwrap()))).unwrap(),
            tiny
        );
        let small = String::from("string".repeat(10));
        assert_eq!(
            String::try_from(Arc::new(Mutex::new(
                small.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            small
        );
        let medium = String::from("string".repeat(1000));
        assert_eq!(
            String::try_from(Arc::new(Mutex::new(
                medium.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            medium
        );
        let large = String::from("string".repeat(100_000));
        assert_eq!(
            String::try_from(Arc::new(Mutex::new(
                large.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            large
        );
        let special = String::from("En å flöt över ängen");
        assert_eq!(
            String::try_from(Arc::new(Mutex::new(
                special.clone().try_into_bytes().unwrap()
            )))
            .unwrap(),
            special
        );
    }
}
