use std::time::Duration as StdDuration;

use crate::impl_try_from_value;
use crate::value::Duration;

impl From<StdDuration> for Duration {
    fn from(duration: StdDuration) -> Self {
        // This fits in an i64 because u64::max_value() / (3600 * 24) < i64::max_value()
        let days = (duration.as_secs() / (3600 * 24)) as i64;
        // This fits in an i64 since it will be less than 3600 * 24
        let seconds = (duration.as_secs() % (3600 * 24)) as i64;
        // This fits in an i32 because 0 <= nanos < 1e9 which is less than i32::max_value()
        let nanos = duration.subsec_nanos() as i32;

        // Months are not well-defined in terms of seconds so let's not use them here
        Self {
            months: 0,
            days,
            seconds,
            nanos,
        }
    }
}

// We cannot convert to std::time::Duration, since months are not well-defined in terms of seconds, and our Duration can
// hold quantities that are impossible to hold in a std::time::Duration (like negative durations).

impl_try_from_value!(Duration, Duration);
