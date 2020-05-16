use chrono::{DateTime, Offset, TimeZone, Timelike};

use crate::value::DateTimeOffset;

impl<T: TimeZone> From<DateTime<T>> for DateTimeOffset {
    fn from(date_time: DateTime<T>) -> Self {
        Self {
            epoch_seconds: date_time.timestamp(),
            nanos: date_time.nanosecond() as i64,
            offset_seconds: date_time.offset().fix().local_minus_utc(),
        }
    }
}
