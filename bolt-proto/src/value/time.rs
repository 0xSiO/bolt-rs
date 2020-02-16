use std::convert::TryFrom;

use chrono::{DateTime, FixedOffset, Timelike};

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x54;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct Time {
    pub(crate) nanos_since_midnight: i64,
    pub(crate) zone_offset: i32,
}

impl From<DateTime<FixedOffset>> for Time {
    fn from(date_time: DateTime<FixedOffset>) -> Self {
        Self {
            nanos_since_midnight: date_time.num_seconds_from_midnight() as i64 * 1_000_000_000,
            zone_offset: date_time.offset().local_minus_utc(),
        }
    }
}

impl TryFrom<Value> for Time {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Time(time) => Ok(time),
            _ => Err(ValueError::InvalidConversion(value).into()),
        }
    }
}
