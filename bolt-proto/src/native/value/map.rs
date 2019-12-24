use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

use failure::Error;

use crate::bolt::value::{Map, Value};
use crate::error::ValueError;

impl<K, V> TryInto<HashMap<K, V>> for Map
where
    K: Hash + Eq + TryFrom<Value, Error = Error>,
    V: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<HashMap<K, V>, Self::Error> {
        let mut map: HashMap<K, V> = HashMap::with_capacity(self.value.len());
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

    fn try_into(self) -> Result<HashMap<K, V>, Self::Error> {
        match self {
            Value::Map(map) => Ok(map.try_into()?),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}
