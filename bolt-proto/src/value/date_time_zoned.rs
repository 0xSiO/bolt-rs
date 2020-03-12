use std::convert::TryFrom;

use chrono::{Duration, NaiveDate};

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x44;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeZoned {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) zone_id: String,
}

// TODO
// impl<T: TimeZone> From<DateTime<T>> for Time {
//     fn from(date_time: DateTime<T>) -> Self {}
// }

#[cfg(test)]
mod tests {}
