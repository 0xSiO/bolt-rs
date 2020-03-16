pub use message::Message;
pub use serialization::{Deserialize, Marker, Serialize, Signature};
pub use value::Value;

pub mod error;
pub mod message;
mod serialization;
pub mod value;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_try_from_value {
    ($T:path, $V:ident) => {
        impl ::std::convert::TryFrom<$crate::Value> for $T {
            type Error = $crate::error::Error;

            fn try_from(value: $crate::Value) -> $crate::error::Result<Self> {
                match value {
                    $crate::Value::$V(inner) => Ok(inner),
                    _ => Err($crate::error::ConversionError::FromValue(value).into()),
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_try_from_message {
    ($T:path, $V:ident) => {
        impl ::std::convert::TryFrom<$crate::Message> for $T {
            type Error = $crate::error::Error;

            fn try_from(message: $crate::Message) -> $crate::error::Result<Self> {
                match message {
                    $crate::Message::$V(inner) => Ok(inner),
                    _ => Err($crate::error::ConversionError::FromMessage(message).into()),
                }
            }
        }
    };
}
