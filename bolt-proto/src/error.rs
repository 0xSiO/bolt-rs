use std::string::FromUtf8Error;

use thiserror::Error;

use crate::{Message, Value};

pub type MarkerResult<T> = std::result::Result<T, MarkerError>;
pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
pub type SerializeResult<T> = std::result::Result<T, SerializationError>;
pub type DeserializeResult<T> = std::result::Result<T, DeserializationError>;

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
    // TODO: Remove / refactor?
    #[error(transparent)]
    MarkerError(#[from] MarkerError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("invalid conversion from value {0:?}")]
    FromValue(Value),
    #[error("invalid conversion from message {0:?}")]
    FromMessage(Message),
}

#[derive(Debug, Error)]
pub enum MarkerError {
    #[error("value too large (size: {0})")]
    ValueTooLarge(usize),
}

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error(transparent)]
    MarkerError(#[from] MarkerError),
}

#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("panicked during deserialization")]
    Panicked,
    #[error("invalid marker byte: {0:x}")]
    InvalidMarkerByte(u8),
    #[error("invalid signature byte: {0:x}")]
    InvalidSignatureByte(u8),
    #[error("invalid size ({size} fields) for signature byte {signature:x}")]
    InvalidSize { size: usize, signature: u8 },
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error("string deserialization failed: {0}")]
    InvalidUTF8(#[from] FromUtf8Error),
}
