use failure::Fail;

use crate::bolt::Message;
use crate::bolt::Value;

#[derive(Debug, Fail)]
pub(crate) enum ValueError {
    #[fail(display = "Value too large (length {})", _0)]
    TooLarge(usize),
    #[fail(display = "Invalid conversion from {:?}", _0)]
    InvalidConversion(Value),
}

#[derive(Debug, Fail)]
pub(crate) enum MessageError {
    #[fail(display = "Invalid conversion from {:?}", _0)]
    InvalidConversion(Message),
}

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub(crate) struct SerializeError(pub(crate) String);

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub(crate) struct DeserializeError(pub(crate) String);
