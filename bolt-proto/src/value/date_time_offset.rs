use std::convert::TryFrom;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};

use bolt_proto_derive::*;

use crate::error::*;
use crate::Value;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x46;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeOffset {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) offset_seconds: i32,
}

impl DateTimeOffset {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
        zone_offset: (i32, i32),
    ) -> Result<Self> {
        let date_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(ValueError::InvalidDate(year, month, day))?,
            NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond).ok_or(
                ValueError::InvalidTime(hour, minute, second, nanosecond, zone_offset),
            )?,
        );
        Ok(Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            offset_seconds: zone_offset.0 * 3600 + zone_offset.1 * 60,
        })
    }
}

// TODO
// impl<T: TimeZone> From<DateTime<T>> for DateTimeOffset {
//     fn from(date_time: DateTime<T>) -> Self {}
// }

#[cfg(test)]
mod tests {}
