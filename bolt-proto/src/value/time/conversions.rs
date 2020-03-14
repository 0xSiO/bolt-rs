use chrono::{DateTime, Offset, TimeZone, Timelike};

use crate::impl_try_from_value;
use crate::value::Time;

impl<T: TimeZone> From<DateTime<T>> for Time {
    fn from(date_time: DateTime<T>) -> Self {
        Self {
            nanos_since_midnight: date_time.num_seconds_from_midnight() as i64 * 1_000_000_000,
            zone_offset: date_time.offset().fix().local_minus_utc(),
        }
    }
}

impl_try_from_value!(Time, Time);
