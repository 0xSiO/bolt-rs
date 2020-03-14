use chrono::{NaiveTime, Timelike};

use crate::impl_try_from_value;
use crate::value::LocalTime;

impl From<NaiveTime> for LocalTime {
    fn from(naive_time: NaiveTime) -> Self {
        Self {
            nanos_since_midnight: naive_time.num_seconds_from_midnight() as i64 * 1_000_000_000,
        }
    }
}

impl_try_from_value!(LocalTime, LocalTime);
