use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use bytes::Bytes;

pub type SerializeResult<T> = Result<T, SerializeError>;

pub trait Serialize {
    fn get_marker(&self) -> SerializeResult<u8>;

    fn try_into_bytes(self) -> SerializeResult<Bytes>
    where
        Self: TryInto<Bytes, Error = SerializeError>,
    {
        self.try_into()
    }
}

impl Serialize for Box<dyn Serialize> {
    fn get_marker(&self) -> SerializeResult<u8> {
        self.deref().get_marker()
    }
}

impl TryInto<Bytes> for Box<dyn Serialize> {
    type Error = SerializeError;

    fn try_into(self) -> SerializeResult<Bytes> {
        self.try_into_bytes()
    }
}

#[derive(Debug)]
pub struct SerializeError {
    message: String,
}

impl SerializeError {
    pub fn new(message: String) -> Self {
        SerializeError { message }
    }
}

impl Display for SerializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for SerializeError {}
