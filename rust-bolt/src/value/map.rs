use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;

use bytes::{BufMut, Bytes, BytesMut};
use failure::Error;

use crate::error::ValueError;
use crate::serialize::Serialize;
use crate::value::Marker;

const MARKER_TINY: u8 = 0xA0;
const MARKER_SMALL: u8 = 0xD8;
const MARKER_MEDIUM: u8 = 0xD9;
const MARKER_LARGE: u8 = 0xDA;

#[derive(Debug)]
pub struct Map<K, V>
where
    // TODO: Waiting for trait aliases https://github.com/rust-lang/rust/issues/41517
    K: Marker + Serialize + Hash + Eq,
    V: Marker + Serialize,
{
    // TODO: Maps permit a mixture of types, use an enum for Value types
    pub(crate) value: HashMap<K, V>,
}

impl<K, V, X, Y> From<HashMap<K, V>> for Map<X, Y>
where
    K: Into<X>,
    V: Into<Y>,
    X: Marker + Serialize + Hash + Eq,
    Y: Marker + Serialize,
{
    fn from(value: HashMap<K, V, RandomState>) -> Self {
        Self {
            value: value
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

impl<K, V> Marker for Map<K, V>
where
    K: Marker + Serialize + Hash + Eq,
    V: Marker + Serialize,
{
    fn get_marker(&self) -> Result<u8, Error> {
        match self.value.len() {
            0..=15 => Ok(MARKER_TINY | self.value.len() as u8),
            16..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=4_294_967_295 => Ok(MARKER_LARGE),
            _ => Err(ValueError::TooLarge(self.value.len()).into()),
        }
    }
}

impl<K, V> Serialize for Map<K, V>
where
    K: Marker + Serialize + Hash + Eq,
    V: Marker + Serialize,
{
}

impl<K, V> TryInto<Bytes> for Map<K, V>
where
    K: Marker + Serialize + Hash + Eq,
    V: Marker + Serialize,
{
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        let marker = self.get_marker()?;
        // There is no "good" worst case capacity to use here
        let mut bytes = BytesMut::with_capacity(self.value.len());
        bytes.put_u8(marker);
        match self.value.len() {
            0..=15 => {}
            16..=255 => bytes.put_u8(self.value.len() as u8),
            256..=65_535 => bytes.put_u16(self.value.len() as u16),
            65_536..=4_294_967_295 => bytes.put_u32(self.value.len() as u32),
            _ => return Err(ValueError::TooLarge(self.value.len()).into()),
        }
        for (key, value) in self.value {
            bytes.put(&mut key.try_into_bytes().unwrap());
            bytes.put(&mut value.try_into_bytes().unwrap());
        }
        Ok(bytes.freeze())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::FromIterator;

    use bytes::Bytes;

    use crate::serialize::Serialize;
    use crate::value::{Integer, Marker, String};

    use super::{Map, MARKER_SMALL, MARKER_TINY};

    #[test]
    fn get_marker() {
        let empty_map: Map<String, Integer> = HashMap::<&str, i8>::new().into();
        assert_eq!(empty_map.get_marker().unwrap(), MARKER_TINY);
        let tiny_map: Map<String, Integer> =
            HashMap::<&str, i8>::from_iter(vec![("a", 1_i8), ("b", 2_i8), ("c", 3_i8)]).into();
        assert_eq!(
            tiny_map.get_marker().unwrap(),
            MARKER_TINY | tiny_map.value.len() as u8
        );
    }

    #[test]
    fn try_into_bytes() {
        let empty_map: Map<String, Integer> = HashMap::<&str, i8>::new().into();
        assert_eq!(
            empty_map.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_TINY | 0 as u8])
        );
        let tiny_map: Map<String, Integer> =
            HashMap::<&str, i8>::from_iter(vec![("a", 1_i8)]).into();
        assert_eq!(
            tiny_map.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_TINY | 1, 0x81, 0x61, 0x01])
        );

        let small_map: Map<String, Integer> = HashMap::<&str, i8>::from_iter(vec![
            ("a", 1_i8),
            ("b", 1_i8),
            ("c", 3_i8),
            ("d", 4_i8),
            ("e", 5_i8),
            ("f", 6_i8),
            ("g", 7_i8),
            ("h", 8_i8),
            ("i", 9_i8),
            ("j", 0_i8),
            ("k", 1_i8),
            ("l", 2_i8),
            ("m", 3_i8),
            ("n", 4_i8),
            ("o", 5_i8),
            ("p", 6_i8),
        ])
        .into();
        let small_len = small_map.value.len();
        let small_bytes = small_map.try_into_bytes().unwrap();
        // Can't check the whole map since the bytes are in no particular order, check marker/length instead
        assert_eq!(small_bytes[0], MARKER_SMALL);
        // Marker byte, size (u8), then list of 2-byte String (marker, value) + 1-byte tiny ints
        assert_eq!(small_bytes.len(), 2 + small_len * 3);
    }
}
