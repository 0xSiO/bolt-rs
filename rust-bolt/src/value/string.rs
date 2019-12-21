use std::convert::TryInto;
use std::mem;

use bytes::{BufMut, Bytes, BytesMut};

use crate::serialize::{SerializeError, SerializeResult, Value};

const MARKER_TINY: u8 = 0x80;
const MARKER_SMALL: u8 = 0xD0;
const MARKER_MEDIUM: u8 = 0xD1;
const MARKER_LARGE: u8 = 0xD2;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct String {
    pub(crate) value: std::string::String,
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

impl Value for String {
    fn get_marker(&self) -> SerializeResult<u8> {
        match self.value.len() {
            0..=15 => Ok(MARKER_TINY | self.value.len() as u8),
            16..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=4_294_967_295 => Ok(MARKER_LARGE),
            _ => Err(SerializeError::new(&format!(
                "String length too long: {}",
                self.value.len()
            ))),
        }
    }
}

impl TryInto<Bytes> for String {
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        let marker = self.get_marker()?;
        // Worst case is a large string, with marker byte, 32-bit size value, and length
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() + mem::size_of::<u32>() + self.value.len(),
        );
        bytes.put_u8(marker);
        match self.value.len() {
            0..=15 => {}
            16..=255 => bytes.put_u8(self.value.len() as u8),
            256..=65_535 => bytes.put_u16(self.value.len() as u16),
            65_536..=4_294_967_295 => bytes.put_u32(self.value.len() as u32),
            _ => {
                return Err(SerializeError::new(&format!(
                    "String length too long: {}",
                    self.value.len()
                )));
            }
        }
        bytes.put_slice(self.value.as_bytes());
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::serialize::Value;

    use super::{String, MARKER_LARGE, MARKER_MEDIUM, MARKER_SMALL, MARKER_TINY};

    #[test]
    fn get_marker() {
        let tiny = String::from("string".repeat(1));
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
        let tiny_bytes = String::from("a".to_string()).try_into_bytes().unwrap();
        assert_eq!(tiny_bytes, Bytes::from_static(&[0x81, 0x61]));
        let normal_bytes = String::from("abcdefghijklmnopqrstuvwxyz".to_string())
            .try_into_bytes()
            .unwrap();
        assert_eq!(
            normal_bytes,
            Bytes::from_static(&[
                0xD0, 0x1A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C,
                0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A
            ])
        );
        let special_bytes = String::from("En å flöt över ängen".to_string())
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
}
