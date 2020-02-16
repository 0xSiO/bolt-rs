use std::convert::TryFrom;

use chrono::{DateTime, FixedOffset, NaiveTime, Timelike};

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

impl Time {
    pub fn new(
        hour: u32,
        minute: u32,
        second: u32,
        fraction: u32,
        zone_offset: (i32, i32),
    ) -> Result<Self> {
        let time = NaiveTime::from_hms_nano_opt(hour, minute, second, fraction).ok_or(
            ValueError::InvalidTime(hour, minute, second, fraction, zone_offset),
        )?;
        Ok(Self {
            nanos_since_midnight: time.num_seconds_from_midnight() as i64 * 1_000_000_000,
            zone_offset: zone_offset.0 * 3600 + zone_offset.1 * 60,
        })
    }
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
