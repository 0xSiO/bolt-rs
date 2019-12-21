use std::convert::TryInto;
use std::fmt::Debug;
use std::ops::Deref;

use bytes::Bytes;
use failure::{Error, Fail};

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
#[fail(display = "{}", _0)]
pub struct SerializeError(pub String);

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct DeserializeError(pub String);
