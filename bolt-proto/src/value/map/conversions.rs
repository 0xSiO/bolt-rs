use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;

use crate::error::*;
use crate::value::Map;
use crate::Value;

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

impl<K, V> TryFrom<Value> for HashMap<K, V>
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
    V: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(map) => {
                let mut new_map = HashMap::with_capacity(map.value.len());
                for (k, v) in map.value {
                    new_map.insert(K::try_from(k)?, V::try_from(v)?);
                }
                Ok(new_map)
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl<K> TryFrom<Value> for HashMap<K, Value>
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(map) => {
                let mut new_map = HashMap::with_capacity(map.value.len());
                for (k, v) in map.value {
                    new_map.insert(K::try_from(k)?, v);
                }
                Ok(new_map)
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}
