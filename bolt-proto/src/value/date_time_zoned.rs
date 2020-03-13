use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};
use chrono_tz::Tz;

use bolt_proto_derive::*;

use crate::error::*;
use crate::impl_try_from_value;

pub(crate) const MARKER: u8 = 0xB3;
pub(crate) const SIGNATURE: u8 = 0x66;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Signature, Marker, Serialize, Deserialize)]
pub struct DateTimeZoned {
    pub(crate) epoch_seconds: i64,
    pub(crate) nanos: i64,
    pub(crate) zone_id: String,
}

impl DateTimeZoned {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
        zone_id: String,
    ) -> Result<Self> {
        let date_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, day)
                .ok_or(Error::InvalidDate(year, month, day))?,
            NaiveTime::from_hms_nano_opt(hour, minute, second, nanosecond)
                .ok_or(Error::InvalidTime(hour, minute, second, nanosecond))?,
        );
        let timezone: Tz = zone_id
            .parse()
            .map_err(|_| Error::InvalidTimeZoneId(zone_id))?;
        Ok(Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            zone_id: timezone.name().to_string(),
        })
    }
}

// Can't impl<T: TimeZone> From<DateTime<T>> for DateTimeZoned, since we can't get a timezone name from an Offset

impl From<DateTimeZoned> for DateTime<FixedOffset> {
    fn from(date_time_zoned: DateTimeZoned) -> Self {
        // Time zone guaranteed to be valid in existing objects, ok to unwrap
        let timezone: Tz = date_time_zoned.zone_id.parse().unwrap();
        let timezone: FixedOffset = timezone
            // TODO: Check if any random date works here (it should...)
            .offset_from_utc_date(&NaiveDate::from_ymd(2020, 1, 1))
            .fix();
        timezone.timestamp(date_time_zoned.epoch_seconds, date_time_zoned.nanos as u32)
    }
}

impl From<DateTimeZoned> for DateTime<Tz> {
    fn from(date_time_zoned: DateTimeZoned) -> Self {
        // Time zone guaranteed to be valid in existing objects, ok to unwrap
        let timezone: Tz = date_time_zoned.zone_id.parse().unwrap();
        timezone.timestamp(date_time_zoned.epoch_seconds, date_time_zoned.nanos as u32)
    }
}

impl_try_from_value!(DateTimeZoned, DateTimeZoned);

#[cfg(test)]
mod tests {}
