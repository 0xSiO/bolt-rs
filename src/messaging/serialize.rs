use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;

use bytes::Bytes;

pub type SerializeResult<T> = Result<T, SerializeError>;

pub trait Serialize {
    fn get_marker(&self) -> SerializeResult<u8>;

    fn try_into_bytes(self) -> SerializeResult<Bytes>
        where Self: TryInto<Bytes, Error=SerializeError> {
        self.try_into()
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
