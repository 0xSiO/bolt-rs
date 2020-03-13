use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};

use bolt_proto_derive::*;

use crate::error::*;
use crate::impl_try_from_value;

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
                .ok_or(Error::InvalidDate(year, month, day))?,
            NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond)
                .ok_or(Error::InvalidTime(hour, minute, second, nanosecond))?,
        );
        let offset = FixedOffset::east_opt(zone_offset.0 * 3600 + zone_offset.1 * 60)
            .ok_or(Error::InvalidTimeZoneOffset(zone_offset))?;
        Ok(Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            offset_seconds: offset.local_minus_utc(),
        })
    }
}

impl<T: TimeZone> From<DateTime<T>> for DateTimeOffset {
    fn from(date_time: DateTime<T>) -> Self {
        Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            offset_seconds: date_time.offset().fix().local_minus_utc(),
        }
    }
}

impl From<DateTimeOffset> for DateTime<FixedOffset> {
    fn from(date_time_offset: DateTimeOffset) -> Self {
        FixedOffset::east(date_time_offset.offset_seconds).timestamp(
            date_time_offset.epoch_seconds,
            date_time_offset.nanos as u32,
        )
    }
}

impl_try_from_value!(DateTimeOffset, DateTimeOffset);

#[cfg(test)]
// TODO
mod tests {}
