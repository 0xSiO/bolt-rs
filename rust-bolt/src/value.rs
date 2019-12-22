use std::convert::{TryFrom, TryInto};
use std::hash::Hash;

use bytes::{Buf, Bytes};
use failure::Error;

use crate::serialize::{Deserialize, Serialize};

pub use self::boolean::Boolean;
pub use self::integer::Integer;
pub use self::map::Map;
pub use self::null::Null;
pub use self::string::String;
use crate::error::DeserializeError;
use std::panic::catch_unwind;
use std::sync::Mutex;

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

impl Deserialize<'_> for Value {}

impl TryFrom<&mut Bytes> for Value {
    type Error = Error;

    fn try_from(input_bytes: &mut Bytes) -> Result<Self, Self::Error> {
        let input_bytes = Mutex::new(input_bytes);
        let result: Result<Value, Error> = catch_unwind(move || {
            let input_bytes = input_bytes.lock().unwrap();
            // TODO: Make sure clone() also preserves position of buffer cursor
            let marker = input_bytes.clone().get_u8();

            match marker {
                // TODO: Can't do the below; try_from should take an Arc<Mutex<Bytes>>
                // null::MARKER => Ok(Value::Null(Null::try_from(*input_bytes)?)),
                // boolean::MARKER_FALSE | boolean::MARKER_TRUE => {
                //     Ok(Value::Boolean(Boolean::try_from(*input_bytes)?))
                // }
                _ => todo!(),
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Value from Bytes: {}", err))
        })?)
    }
}
