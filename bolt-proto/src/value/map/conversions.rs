use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

use crate::error::*;
use crate::value::Map;
use crate::Value;

impl<K, V> TryInto<HashMap<K, V>> for Map
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
    V: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<HashMap<K, V>> {
        let mut map = HashMap::with_capacity(self.value.len());
        for (k, v) in self.value {
            map.insert(K::try_from(k)?, V::try_from(v)?);
        }
        Ok(map)
    }
}

impl<K> TryInto<HashMap<K, Value>> for Map
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<HashMap<K, Value>> {
        let mut map = HashMap::with_capacity(self.value.len());
        for (k, v) in self.value {
            map.insert(K::try_from(k)?, v);
        }
        Ok(map)
    }
}

// We don't need TryFrom<Value> for Map since it can be converted directly into a HashMap
// impl_try_from_value!(Map, Map);
