use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;

use bolt_proto::Value;

#[derive(Default)]
pub struct ValueMap {
    pub(crate) value: HashMap<String, Value>,
}

impl<K, V> From<HashMap<K, V>> for ValueMap
where
    K: Into<String>,
    V: Into<Value>,
{
    fn from(map: HashMap<K, V, RandomState>) -> Self {
        Self {
            value: map.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
        }
    }
}

impl<K, V> FromIterator<(K, V)> for ValueMap
where
    K: Eq + Hash + Into<String>,
    V: Into<Value>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            value: HashMap::from_iter(iter.into_iter().map(|(k, v)| (k.into(), v.into()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;

    #[test]
    fn create_metadata() {
        let empty_map: ValueMap = Default::default();
        assert!(empty_map.value.is_empty());
        let value_map = ValueMap::from(HashMap::from_iter(vec![("key", "value")]));
        assert_eq!(
            HashMap::from_iter(vec![(String::from("key"), Value::from("value"))]),
            value_map.value
        );
    }
}
