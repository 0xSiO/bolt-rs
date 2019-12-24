use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use failure::Error;

use crate::bolt::value::Marker;
use crate::error::{DeserializeError, ValueError};
use crate::{Deserialize, Serialize, Value};
use std::mem;

pub(crate) const MARKER_TINY: u8 = 0xA0;
pub(crate) const MARKER_SMALL: u8 = 0xD8;
pub(crate) const MARKER_MEDIUM: u8 = 0xD9;
pub(crate) const MARKER_LARGE: u8 = 0xDA;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Map {
    pub(crate) value: HashMap<Value, Value>,
}

impl Hash for Map {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        panic!("Cannot hash a Map")
    }
}

impl<K, V> From<HashMap<K, V>> for Map
where
    K: Into<Value>,
    V: Into<Value>,
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

impl TryFrom<Value> for Map {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(map) => Ok(map),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}

impl Marker for Map {
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

impl Serialize for Map {}

impl TryInto<Bytes> for Map {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        let marker = self.get_marker()?;
        let mut bytes = BytesMut::with_capacity(mem::size_of::<Value>() * 2 * self.value.len());
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

impl Deserialize for Map {}

impl TryFrom<Arc<Mutex<Bytes>>> for Map {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        let result: Result<Map, Error> = catch_unwind(move || {
            let marker = input_arc.lock().unwrap().get_u8();
            let size = match marker {
                marker if (MARKER_TINY..=(MARKER_TINY | 0x0F)).contains(&marker) => {
                    0x0F & marker as usize
                }
                MARKER_SMALL => input_arc.lock().unwrap().get_u8() as usize,
                MARKER_MEDIUM => input_arc.lock().unwrap().get_u16() as usize,
                MARKER_LARGE => input_arc.lock().unwrap().get_u32() as usize,
                _ => {
                    return Err(
                        DeserializeError(format!("Invalid marker byte: {:x}", marker)).into(),
                    );
                }
            };
            let mut hash_map: HashMap<Value, Value> = HashMap::with_capacity(size);
            for _ in 0..size {
                let key = Value::try_from(Arc::clone(&input_arc))?;
                let value = Value::try_from(Arc::clone(&input_arc))?;
                hash_map.insert(key, value);
            }
            Ok(Map::from(hash_map))
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Map from Bytes: {}", err))
        })?)
    }
}

#[cfg(test)]
mod tests {
    use std::clone::Clone;
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;

    use crate::bolt::value::Marker;
    use crate::Serialize;

    use super::{Map, MARKER_SMALL, MARKER_TINY};

    #[test]
    fn get_marker() {
        let empty_map: Map = HashMap::<&str, i8>::new().into();
        assert_eq!(empty_map.get_marker().unwrap(), MARKER_TINY);
        let tiny_map: Map =
            HashMap::<&str, i8>::from_iter(vec![("a", 1_i8), ("b", 2_i8), ("c", 3_i8)]).into();
        assert_eq!(
            tiny_map.get_marker().unwrap(),
            MARKER_TINY | tiny_map.value.len() as u8
        );
    }

    #[test]
    fn try_into_bytes() {
        let empty_map: Map = HashMap::<&str, i8>::new().into();
        assert_eq!(
            empty_map.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_TINY | 0 as u8])
        );
        let tiny_map: Map = HashMap::<&str, i8>::from_iter(vec![("a", 1_i8)]).into();
        assert_eq!(
            tiny_map.try_into_bytes().unwrap(),
            Bytes::from_static(&[MARKER_TINY | 1, 0x81, 0x61, 0x01])
        );

        let small_map: Map = HashMap::<&str, i8>::from_iter(vec![
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

    #[test]
    fn try_from_bytes() {
        let empty_map: Map = HashMap::<&str, i8>::new().into();
        let empty_map_bytes = empty_map.clone().try_into_bytes().unwrap();
        let tiny_map: Map = HashMap::<&str, i8>::from_iter(vec![("a", 1_i8)]).into();
        let tiny_map_bytes = tiny_map.clone().try_into_bytes().unwrap();
        let small_map: Map = HashMap::<&str, i8>::from_iter(vec![
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
        let small_map_bytes = small_map.clone().try_into_bytes().unwrap();

        assert_eq!(
            Map::try_from(Arc::new(Mutex::new(empty_map_bytes))).unwrap(),
            empty_map
        );
        assert_eq!(
            Map::try_from(Arc::new(Mutex::new(tiny_map_bytes))).unwrap(),
            tiny_map
        );
        assert_eq!(
            Map::try_from(Arc::new(Mutex::new(small_map_bytes))).unwrap(),
            small_map
        );
    }
}
