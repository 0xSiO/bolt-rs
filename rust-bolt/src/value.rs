use std::convert::TryInto;
use std::hash::Hash;

use bytes::Bytes;
use failure::Error;

use crate::serialize::Serialize;

pub use self::boolean::Boolean;
pub use self::integer::Integer;
pub use self::map::Map;
pub use self::null::Null;
pub use self::string::String;

mod boolean;
mod integer;
mod map;
mod null;
mod string;

pub trait Marker {
    fn get_marker(&self) -> Result<u8, Error>;
}

#[derive(Hash, Eq, PartialEq)]
pub enum Value {
    Boolean(Boolean),
    Integer(Integer),
    Map(Map<Value, Value>),
    Null(Null),
    String(String),
}

impl Marker for Value {
    fn get_marker(&self) -> Result<u8, Error> {
        match self {
            Value::Boolean(boolean) => boolean.get_marker(),
            Value::Integer(integer) => integer.get_marker(),
            Value::Map(map) => map.get_marker(),
            Value::Null(null) => null.get_marker(),
            Value::String(string) => string.get_marker(),
        }
    }
}

impl Serialize for Value {}

impl TryInto<Bytes> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        match self {
            Value::Boolean(boolean) => boolean.try_into(),
            Value::Integer(integer) => integer.try_into(),
            Value::Map(map) => map.try_into(),
            Value::Null(null) => null.try_into(),
            Value::String(string) => string.try_into(),
        }
    }
}
