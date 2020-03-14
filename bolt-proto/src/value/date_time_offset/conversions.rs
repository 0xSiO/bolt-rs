use chrono::{DateTime, FixedOffset, Offset, TimeZone, Timelike};

use crate::impl_try_from_value;
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

impl From<DateTimeOffset> for DateTime<FixedOffset> {
    fn from(date_time_offset: DateTimeOffset) -> Self {
        FixedOffset::east(date_time_offset.offset_seconds).timestamp(
            date_time_offset.epoch_seconds,
            date_time_offset.nanos as u32,
        )
    }
}

impl_try_from_value!(DateTimeOffset, DateTimeOffset);
