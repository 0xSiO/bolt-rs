use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

use crate::bolt::value::Map;
use crate::error::*;
use crate::Value;

// Have to use Value for the HashMap values since Value does not impl TryFrom<Value, Error = failure::Error>
// and we need to support HashMaps with Value values (see Node's properties field)
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
