use std::convert::TryFrom;

use chrono::{DateTime, NaiveDateTime, TimeZone, Timelike};
use chrono_tz::Tz;

use crate::error::*;
use crate::value::DateTimeZoned;
use crate::Value;

// Can't impl<T: TimeZone> From<DateTime<T>> for DateTimeZoned, since we can't get a timezone name from an Offset
// Provide separate conversion instead
impl From<(NaiveDateTime, Tz)> for DateTimeZoned {
    fn from(pair: (NaiveDateTime, Tz)) -> Self {
        Self {
            epoch_seconds: pair.0.timestamp(),
            nanos: pair.0.nanosecond() as i64,
            zone_id: pair.1.name().to_string(),
        }
    }
}

impl TryFrom<Value> for DateTime<Tz> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::DateTimeZoned(date_time_zoned) => {
                // Time zone guaranteed to be valid in existing objects, ok to unwrap
                let timezone: Tz = date_time_zoned.zone_id.parse().unwrap();
                Ok(timezone
                    .timestamp_opt(date_time_zoned.epoch_seconds, date_time_zoned.nanos as u32)
                    // epoch_seconds and nanos are guaranteed to be valid in existing objects, ok to unwrap
                    .unwrap())
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

// TODO: Combine this with the conversion from Value::DateTimeOffset
// impl TryFrom<Value> for DateTime<FixedOffset> {
//     type Error = Error;
//
//     fn try_from(value: Value) -> Result<Self> {
//         match value {
//             Value::DateTimeZoned(date_time_zoned) => {
//                 // Time zone guaranteed to be valid in existing objects, ok to unwrap
//                 let timezone: Tz = date_time_zoned.zone_id.value.parse().unwrap();
//                 let timezone: FixedOffset = timezone
//                     // Get the fixed offset (e.g. Pacific Daylight vs. Pacific Standard) for the given point in time
//                     .offset_from_utc_datetime(
//                         &NaiveDateTime::from_timestamp_opt(date_time_zoned.epoch_seconds, 0)
//                             // epoch_seconds is guaranteed to be a valid timestamp, ok to unwrap
//                             .unwrap(),
//                     )
//                     .fix();
//                 Ok(timezone
//                     .timestamp_opt(date_time_zoned.epoch_seconds, date_time_zoned.nanos as u32)
//                     // epoch_seconds and nanos are guaranteed to be valid in existing objects, ok to unwrap
//                     .unwrap())
//             }
//             _ => Err(ConversionError::FromValue(value).into()),
//         }
//     }
// }
