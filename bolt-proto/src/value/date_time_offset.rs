use std::convert::TryFrom;

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x46;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeOffset {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) offset_seconds: i64,
}

// TODO
// impl<T: TimeZone> From<DateTime<T>> for DateTimeOffset {
//     fn from(date_time: DateTime<T>) -> Self {}
// }

#[cfg(test)]
mod tests {}
