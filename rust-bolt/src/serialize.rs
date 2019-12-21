use std::convert::TryInto;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use bytes::Bytes;
use failure::{Error, Fail};

use crate::error::ValueError;

pub type DeserializeResult<T> = Result<T, DeserializeError>;

pub trait Value {
    fn get_marker(&self) -> Result<u8, Error>;

    fn try_into_bytes(self) -> Result<Bytes, Error>
    where
        Self: TryInto<Bytes, Error = Error>,
    {
        self.try_into()
    }
}

impl Value for Box<dyn Value> {
    fn get_marker(&self) -> Result<u8, Error> {
        self.deref().get_marker()
    }
}

impl TryInto<Bytes> for Box<dyn Value> {
    type Error = Error;

    fn try_into(self) -> Result<Bytes, Self::Error> {
        self.try_into_bytes()
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Error during serialization: {}", message)]
pub struct SerializeError {
    message: String,
}

pub type DeserializeError = SerializeError;

impl SerializeError {
    pub fn new(message: &str) -> Self {
        SerializeError {
            message: message.to_string(),
        }
    }
}
