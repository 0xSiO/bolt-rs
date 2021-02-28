use std::convert::{TryFrom, TryInto};
use std::mem;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::*;
use crate::serialization::*;

pub(crate) const MARKER_SMALL: u8 = 0xCC;
pub(crate) const MARKER_MEDIUM: u8 = 0xCD;
pub(crate) const MARKER_LARGE: u8 = 0xCE;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ByteArray {
    pub(crate) value: Vec<u8>,
}

impl Marker for ByteArray {
    fn get_marker(&self) -> Result<u8> {
        match self.value.len() {
            0..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=2_147_483_647 => Ok(MARKER_LARGE),
            _ => Err(Error::ValueTooLarge(self.value.len())),
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
            65_536..=2_147_483_647 => bytes.put_u32(self.value.len() as u32),
            _ => return Err(Error::ValueTooLarge(self.value.len())),
        }
        bytes.put_slice(&self.value);
        Ok(bytes.freeze())
    }
}

impl Deserialize for ByteArray {}

impl TryFrom<Arc<Mutex<Bytes>>> for ByteArray {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self> {
        catch_unwind(move || {
            let mut input_bytes = input_arc.lock().unwrap();
            let marker = input_bytes.get_u8();
            let size = match marker {
                MARKER_SMALL => input_bytes.get_u8() as usize,
                MARKER_MEDIUM => input_bytes.get_u16() as usize,
                MARKER_LARGE => input_bytes.get_u32() as usize,
                _ => {
                    return Err(DeserializationError::InvalidMarkerByte(marker).into());
                }
            };
            let mut byte_arr = vec![0; size];
            input_bytes.copy_to_slice(&mut byte_arr);
            Ok(ByteArray::from(byte_arr))
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

impl From<Vec<u8>> for ByteArray {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

#[cfg(test)]
mod tests {
    use crate::value::*;

    use super::*;

    #[test]
    fn get_marker() {
        let empty_arr: ByteArray = Vec::<u8>::new().into();
        let small_arr: ByteArray = vec![0; 100].into();
        let medium_arr: ByteArray = vec![0; 1000].into();
        assert_eq!(empty_arr.get_marker().unwrap(), MARKER_SMALL);
        assert_eq!(small_arr.get_marker().unwrap(), MARKER_SMALL);
        assert_eq!(medium_arr.get_marker().unwrap(), MARKER_MEDIUM);
    }

    #[test]
    fn get_marker_large() {
        let large_arr: ByteArray = vec![0; 100_000].into();
        assert_eq!(large_arr.get_marker().unwrap(), MARKER_LARGE);
    }

    #[test]
    fn try_into_bytes() {
        let empty_arr: ByteArray = Vec::<u8>::new().into();
        let small_arr: ByteArray = vec![1_u8; 100].into();
        let medium_arr: ByteArray = vec![99_u8; 1000].into();
        assert_eq!(
            empty_arr.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_SMALL, 0_u8])
        );
        let small_arr_expected_bytes: Vec<u8> = vec![MARKER_SMALL, 0x64] // marker, size
            .into_iter()
            .chain(vec![1_u8; 100])
            .collect();
        assert_eq!(
            small_arr.try_into_bytes().unwrap(),
            Bytes::from(small_arr_expected_bytes)
        );
        let medium_arr_expected_bytes: Vec<u8> = vec![MARKER_MEDIUM, 0x03, 0xE8] // marker, size
            .into_iter()
            .chain(vec![99_u8; 1000])
            .collect();
        assert_eq!(
            medium_arr.try_into_bytes().unwrap(),
            Bytes::from(medium_arr_expected_bytes)
        );
    }

    #[test]
    fn try_into_large_bytes() {
        let large_arr: ByteArray = vec![1_u8; 100_000].into();
        let large_arr_expected_bytes: Vec<u8> = vec![MARKER_LARGE, 0x00, 0x01, 0x86, 0xA0] // marker, size
            .into_iter()
            .chain(vec![1_u8; 100_000])
            .collect();
        assert_eq!(
            large_arr.try_into_bytes().unwrap(),
            Bytes::from(large_arr_expected_bytes)
        );
    }

    #[test]
    fn try_from_bytes() {
        let empty_arr: ByteArray = Vec::<u8>::new().into();
        let empty_arr_bytes = empty_arr.clone().try_into_bytes().unwrap();
        let tiny_arr: ByteArray = vec![25_u8; 10].into();
        let tiny_arr_bytes = tiny_arr.clone().try_into_bytes().unwrap();
        let small_arr: ByteArray = vec![1_u8; 100].into();
        let small_arr_bytes = small_arr.clone().try_into_bytes().unwrap();
        let medium_arr: ByteArray = vec![99_u8; 1000].into();
        let medium_arr_bytes = medium_arr.clone().try_into_bytes().unwrap();
        assert_eq!(
            ByteArray::try_from(Arc::new(Mutex::new(empty_arr_bytes))).unwrap(),
            empty_arr
        );
        assert_eq!(
            ByteArray::try_from(Arc::new(Mutex::new(tiny_arr_bytes))).unwrap(),
            tiny_arr
        );
        assert_eq!(
            ByteArray::try_from(Arc::new(Mutex::new(small_arr_bytes))).unwrap(),
            small_arr
        );
        assert_eq!(
            ByteArray::try_from(Arc::new(Mutex::new(medium_arr_bytes))).unwrap(),
            medium_arr
        );
    }

    #[test]
    fn try_from_large_bytes() {
        let large_arr: ByteArray = vec![1_u8; 100_000].into();
        let large_arr_bytes = large_arr.clone().try_into_bytes().unwrap();
        assert_eq!(
            ByteArray::try_from(Arc::new(Mutex::new(large_arr_bytes))).unwrap(),
            large_arr
        );
    }
}
