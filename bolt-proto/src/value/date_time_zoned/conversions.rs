use chrono::{NaiveDateTime, Timelike};
use chrono_tz::Tz;

use crate::value::DateTimeZoned;

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
