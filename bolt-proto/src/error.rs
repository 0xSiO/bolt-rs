use std::string::FromUtf8Error;

use thiserror::Error;

use crate::{Message, Value};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("value too large (size: {0})")]
    ValueTooLarge(usize),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("invalid conversion from value {0:?}")]
    FromValue(Value),
    #[error("invalid conversion from message {0:?}")]
    FromMessage(Message),
}

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("panicked during deserialization")]
    Panicked,
    #[error("invalid marker byte: {0:x}")]
    InvalidMarkerByte(u8),
    #[error("invalid signature byte: {0:x}")]
    InvalidSignatureByte(u8),
    #[error("string deserialization failed: {0}")]
    InvalidUTF8(#[from] FromUtf8Error),
}
