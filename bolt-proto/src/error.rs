use std::str::Utf8Error;

use thiserror::Error;

use crate::{Message, Value};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Value too large (size: {0})")]
    ValueTooLarge(usize),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Invalid conversion from value {0:?}")]
    FromValue(Value),
    #[error("Invalid conversion from message {0:?}")]
    FromMessage(Message),
}

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("Panicked during deserialization")]
    Panicked,
    #[error("Invalid marker byte: {0:x}")]
    InvalidMarkerByte(u8),
    #[error("Invalid signature byte: {0:x}")]
    InvalidSignatureByte(u8),
    #[error("String deserialization failed: {0}")]
    InvalidUTF8(#[from] Utf8Error),
}
