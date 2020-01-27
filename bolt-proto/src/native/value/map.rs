use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

use crate::bolt::value::Map;
use crate::error::*;
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

impl<K, V> TryInto<HashMap<K, V>> for Value
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
    V: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<HashMap<K, V>> {
        match self {
            Value::Map(map) => Ok(map.try_into()?),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
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

impl<K> TryInto<HashMap<K, Value>> for Value
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<HashMap<K, Value>> {
        match self {
            Value::Map(map) => Ok(map.try_into()?),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}
