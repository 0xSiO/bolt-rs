use thiserror::Error;

use crate::{Message, Value};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Value too large (length {0})")]
    ValueTooLarge(usize),
    #[error("Invalid conversion from value {0:?}")]
    InvalidValueConversion(Value),
    #[error("Invalid conversion from message {0:?}")]
    InvalidMessageConversion(Message),
    #[error("Invalid date: {0}-{1}-{2}")]
    InvalidDate(i32, u32, u32),
    #[error("Invalid time: {0}:{1}:{2}:{3} offset {4:?}")]
    InvalidTime(u32, u32, u32, u32, (i32, i32)),
    #[error("{0}")]
    DeserializationFailed(String),
}
