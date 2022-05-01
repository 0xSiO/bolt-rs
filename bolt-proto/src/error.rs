use std::fmt::Display;

use thiserror::Error;

use crate::{Message, Value};

pub type Result<T> = std::result::Result<T, Error>;
pub type ConversionResult<T> = std::result::Result<T, ConversionError>;
pub type SerializeResult<T> = std::result::Result<T, SerializationError>;
pub type DeserializeResult<T> = std::result::Result<T, DeserializationError>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error(transparent)]
    SerializationError(#[from] SerializationError),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("invalid conversion from value {0:?}")]
    FromValue(Value),
    #[error("invalid conversion from message {0:?}")]
    FromMessage(Message),
    #[error("failed deserialization {0}")]
    Serde(String),
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
}

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("value too large (size: {0})")]
    ValueTooLarge(usize),
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
    #[error("string deserialization failed: {0}")]
    InvalidUTF8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    ConversionError(#[from] ConversionError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl serde::de::Error for Error {
    #[cold]
    fn custom<T: Display>(msg: T) -> Error {
        make_error(msg.to_string())
    }

    #[cold]
    fn invalid_type<'a>(unexp: serde::de::Unexpected<'a>, exp: &dyn serde::de::Expected) -> Self {
        if let serde::de::Unexpected::Unit = unexp {
            serde::de::Error::custom(format_args!("invalid type: null, expected {}", exp))
        } else {
            serde::de::Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
        }
    }
}
fn make_error(mut msg: String) -> Error {
    Error::ConversionError(ConversionError::Serde(msg))
}
