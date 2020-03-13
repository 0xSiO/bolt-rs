use std::str::Utf8Error;

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
    // TODO: Remove
    DeserializationFailed(String),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
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
