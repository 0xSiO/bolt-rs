use std::convert::{TryFrom, TryInto};
use std::mem;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::v1::error::*;
use crate::v1::serialization::*;

mod conversions;

pub(crate) const MARKER_SMALL: u8 = 0xCC;
pub(crate) const MARKER_MEDIUM: u8 = 0xCD;
pub(crate) const MARKER_LARGE: u8 = 0xCE;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ByteArray {
    pub(crate) value: Vec<u8>,
}

impl Marker for ByteArray {
    fn get_marker(&self) -> Result<u8> {
        match self.value.len() {
            0..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=4_294_967_295 => Ok(MARKER_LARGE),
            _ => Err(ValueError::TooLarge(self.value.len()).into()),
        }
    }
}

impl Serialize for ByteArray {}

impl TryInto<Bytes> for ByteArray {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        // Worst case is a large ByteArray, with marker byte, 32-bit size value, and length
        let mut bytes = BytesMut::with_capacity(
            mem::size_of::<u8>() + mem::size_of::<u32>() + self.value.len(),
        );
        bytes.put_u8(self.get_marker()?);
        match self.value.len() {
            0..=255 => bytes.put_u8(self.value.len() as u8),
            256..=65_535 => bytes.put_u16(self.value.len() as u16),
            65_536..=4_294_967_295 => bytes.put_u32(self.value.len() as u32),
            _ => return Err(ValueError::TooLarge(self.value.len()).into()),
        }
        bytes.put_slice(&self.value);
        Ok(bytes.freeze())
    }
}

impl Deserialize for ByteArray {}

impl TryFrom<Arc<Mutex<Bytes>>> for ByteArray {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self> {
        let result: Result<ByteArray> = catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
            let marker = input_bytes.get_u8();
            let size = match marker {
                MARKER_SMALL => input_bytes.get_u8() as usize,
                MARKER_MEDIUM => input_bytes.get_u16() as usize,
                MARKER_LARGE => input_bytes.get_u32() as usize,
                _ => {
                    return Err(
                        DeserializeError(format!("Invalid marker byte: {:x}", marker)).into(),
                    );
                }
            };
            let mut byte_arr = BytesMut::with_capacity(size);
            // We resize here so that the length of byte_arr is nonzero, which allows us to use copy_to_slice
            byte_arr.resize(size, 0);
            input_bytes.copy_to_slice(&mut byte_arr);
            Ok(ByteArray::from(byte_arr.as_ref()))
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating ByteArray from Bytes: {}", err))
        })?)
    }
}

// TODO: Tests
