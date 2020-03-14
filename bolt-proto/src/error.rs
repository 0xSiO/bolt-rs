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
    #[error("Overflow encountered")]
    Overflow,
    #[error("Invalid conversion from value {0:?}")]
    InvalidValueConversion(Value), // TODO: Remove
    #[error("Invalid conversion from message {0:?}")]
    InvalidMessageConversion(Message), // TODO: Remove
    #[error("Invalid date: {0}-{1}-{2}")]
    InvalidDate(i32, u32, u32),
    #[error("Invalid time: {0}:{1}:{2}:{3}")]
    InvalidTime(u32, u32, u32, u32),
    #[error("Invalid time zone offset: {0:?}")]
    InvalidTimeZoneOffset((i32, i32)),
    #[error("Invalid time zone ID: {0}")]
    InvalidTimeZoneId(String),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Panicked during conversion")]
    Panicked,
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
