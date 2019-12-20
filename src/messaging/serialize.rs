use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;

use bytes::{Bytes, BytesMut};

pub type MarkerResult = Result<u8, SerializeError>;
pub type BytesResult = Result<Bytes, SerializeError>;
pub type BytesMutResult = Result<BytesMut, SerializeError>;

pub trait Serialize {
    fn get_marker(&self) -> MarkerResult;

    fn into_bytes(self) -> BytesResult
        where Self: TryInto<Bytes, Error=SerializeError> {
        self.try_into()
    }

    fn into_bytes_mut(self) -> BytesMutResult
        where Self: TryInto<BytesMut, Error=SerializeError> {
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
