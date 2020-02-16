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
    #[fail(display = "Invalid date: {}/{}/{}", _0, _1, _2)]
    InvalidDate(i32, u32, u32),
    #[fail(display = "Invalid time: {}:{}:{}:{} offset {:?}", _0, _1, _2, _3, _4)]
    InvalidTime(u32, u32, u32, u32, (i32, i32)),
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
