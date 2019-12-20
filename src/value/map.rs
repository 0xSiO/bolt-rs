use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::TryInto;

use bytes::{BufMut, Bytes, BytesMut};

use crate::serialize::{Serialize, SerializeError, SerializeResult};

const MARKER_TINY: u8 = 0xA0;
const MARKER_SMALL: u8 = 0xD8;
const MARKER_MEDIUM: u8 = 0xD9;
const MARKER_LARGE: u8 = 0xDA;

struct Map<K, V>
    where K: Serialize, V: Serialize {
    value: HashMap<K, V>,
}

impl<K, V> From<HashMap<K, V>> for Map<K, V>
    where K: Serialize, V: Serialize {
    fn from(value: HashMap<K, V, RandomState>) -> Self {
        Self { value }
    }
}

impl<K, V> Serialize for Map<K, V>
    where K: Serialize, V: Serialize {
    fn get_marker(&self) -> SerializeResult<u8> {
        match self.value.len() {
            0..=15 => Ok(MARKER_TINY | self.value.len() as u8),
            16..=255 => Ok(MARKER_SMALL),
            256..=65_535 => Ok(MARKER_MEDIUM),
            65_536..=4_294_967_295 => Ok(MARKER_LARGE),
            _ => Err(SerializeError::new(format!("Too many pairs in Map: {}", self.value.len())))
        }
    }
}

impl<K, V> TryInto<Bytes> for Map<K, V>
    where K: Serialize + TryInto<Bytes, Error=SerializeError>,
          V: Serialize + TryInto<Bytes, Error=SerializeError> {
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        let marker = self.get_marker()?;
        // There is no "good" worst case capacity to use here
        let mut bytes = BytesMut::with_capacity(self.value.len());
        bytes.put_u8(marker);
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

    use crate::serialize::Serialize;
    use crate::value::{Boolean, String};

    use super::{Map, MARKER_LARGE, MARKER_MEDIUM, MARKER_SMALL, MARKER_TINY};

    #[test]
    fn get_marker() {
        let empty_map: Map<String, String> = HashMap::new().into();
        assert_eq!(empty_map.get_marker().unwrap(), MARKER_TINY);
        let tiny_map: Map<String, Boolean> = HashMap::from_iter(vec![("a", true), ("b", false), ("c", true)]
            .into_iter().map(|(k, v)| (k.to_string().into(), v.into()))).into();
        assert_eq!(tiny_map.get_marker().unwrap(), MARKER_TINY | tiny_map.value.len() as u8);
    }
}
