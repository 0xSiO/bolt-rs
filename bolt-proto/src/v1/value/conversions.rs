use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

use crate::v1::error::*;
use crate::v1::value::*;

// ----------------------- FROM -----------------------

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::v1::Value {
                fn from(value: $T) -> Self {
                    Value::Integer(Integer::from(value))
                }
            }
        )*
    };
}
impl_from_int!(i8, i16, i32, i64);

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        Value::List(List::from(value))
    }
}

impl<K, V> From<HashMap<K, V>> for Value
where
    K: Into<Value>,
    V: Into<Value>,
{
    fn from(value: HashMap<K, V, RandomState>) -> Self {
        Value::Map(Map::from(value))
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(std::string::String::from(value))
    }
}

impl From<std::string::String> for Value {
    fn from(value: std::string::String) -> Self {
        Value::String(value)
    }
}

impl From<Node> for Value {
    fn from(value: Node) -> Self {
        Value::Node(value)
    }
}

impl From<Relationship> for Value {
    fn from(value: Relationship) -> Self {
        Value::Relationship(value)
    }
}

impl From<Path> for Value {
    fn from(value: Path) -> Self {
        Value::Path(value)
    }
}

impl From<UnboundRelationship> for Value {
    fn from(value: UnboundRelationship) -> Self {
        Value::UnboundRelationship(value)
    }
}

// ----------------------- INTO -----------------------

impl<T> TryInto<Vec<T>> for Value
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_into(self) -> Result<Vec<T>> {
        match self {
            Value::List(list) => list.try_into(),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
    }
}

impl TryInto<Vec<Value>> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Value>> {
        match self {
            Value::List(list) => list.try_into(),
            _ => Err(ValueError::InvalidConversion(self).into()),
        }
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
