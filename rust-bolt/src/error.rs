use failure::Fail;

use crate::bolt::value::BoltValue;

#[derive(Debug, Fail)]
pub enum ValueError {
    #[fail(display = "Value too large (length {})", _0)]
    TooLarge(usize),
    #[fail(display = "Invalid conversion from {:?}", _0)]
    InvalidConversion(BoltValue),
}

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct SerializeError(pub String);

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct DeserializeError(pub String);
