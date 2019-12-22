use std::convert::{TryFrom, TryInto};
use std::hash::Hash;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::{Buf, Bytes};
use failure::Error;

use crate::error::DeserializeError;
use crate::serialize::{Deserialize, Serialize};

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

#[derive(Debug, Hash, Eq, PartialEq)]
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

impl Deserialize for Value {}

impl TryFrom<Arc<Mutex<Bytes>>> for Value {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self, Self::Error> {
        let result: Result<Value, Error> = catch_unwind(move || {
            let input_bytes = input_arc.lock().unwrap();
            // TODO: Make sure clone() also preserves position of buffer cursor
            let marker = input_bytes.clone().get_u8();

            match marker {
                null::MARKER => Ok(Value::Null(Null::try_from(Arc::clone(&input_arc))?)),
                boolean::MARKER_FALSE | boolean::MARKER_TRUE => {
                    Ok(Value::Boolean(Boolean::try_from(Arc::clone(&input_arc))?))
                }
                _ => todo!(),
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Value from Bytes: {}", err))
        })?)
    }
}
