use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};

use bolt_proto_derive::*;

use crate::error::*;

mod conversions;

pub(crate) const MARKER: u8 = 0xB2;
pub(crate) const SIGNATURE: u8 = 0x64;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct LocalDateTime {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
}

impl LocalDateTime {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nano: u32,
    ) -> Result<Self> {
        let date_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(Error::InvalidDate(year, month, day))?,
            NaiveTime::from_hms_nano_opt(hour, minute, second, nano)
                .ok_or(Error::InvalidTime(hour, minute, second, nano))?,
        );
        Ok(Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
        })
    }
}

#[cfg(test)]
// TODO
mod tests {}
