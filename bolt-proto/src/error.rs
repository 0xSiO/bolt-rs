use failure::Fail;

use crate::{Message, Value};

pub(crate) type Error = failure::Error;
pub(crate) type Result<T> = failure::Fallible<T>;

#[derive(Debug, Fail)]
pub enum ValueError {
    #[fail(display = "Value too large (length {})", _0)]
    TooLarge(usize),
    #[fail(display = "Invalid conversion from {:?}", _0)]
    InvalidConversion(Value),
}

#[derive(Debug, Fail)]
pub enum MessageError {
    #[fail(display = "Invalid conversion from {:?}", _0)]
    InvalidConversion(Message),
}

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct SerializeError(pub(crate) String);

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct DeserializeError(pub(crate) String);
