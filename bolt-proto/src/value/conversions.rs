use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::BuildHasher;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone};
use chrono_tz::Tz;

use crate::error::*;
use crate::value::*;

// ------------------------- Into Value -------------------------

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl From<$T> for $crate::Value {
                fn from(value: $T) -> Self {
                    Value::Integer(value as i64)
                }
            }
        )*
    };
}
impl_from_int!(i8, i16, i32, i64);

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Value::Bytes(value.to_vec())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(value)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        Value::List(value.into_iter().map(T::into).collect())
    }
}

impl<K, V, S> From<HashMap<K, V, S>> for Value
where
    K: Into<std::string::String>,
    V: Into<Value>,
    S: BuildHasher,
{
    fn from(value: HashMap<K, V, S>) -> Self {
        Value::Map(
            value
                .into_iter()
                .map(|(k, v)| (K::into(k), V::into(v)))
                .collect(),
        )
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(String::from(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<Node> for Value {
    fn from(value: Node) -> Self {
        Value::Node(value)
    }
}

impl From<Relationship> for Value {
    fn from(value: Relationship) -> Self {
        Value::Relationship(value)
    }
}

impl From<Path> for Value {
    fn from(value: Path) -> Self {
        Value::Path(value)
    }
}

impl From<UnboundRelationship> for Value {
    fn from(value: UnboundRelationship) -> Self {
        Value::UnboundRelationship(value)
    }
}

impl From<NaiveDate> for Value {
    fn from(value: NaiveDate) -> Self {
        Value::Date(value)
    }
}

// No timezone-aware time in chrono, so provide a separate conversion
impl<O: Offset> From<(NaiveTime, O)> for Value {
    fn from(pair: (NaiveTime, O)) -> Self {
        Value::Time(pair.0, pair.1.fix())
    }
}

impl<T: TimeZone> From<DateTime<T>> for Value {
    fn from(value: DateTime<T>) -> Self {
        Value::DateTimeOffset(DateTimeOffset::from(value))
    }
}

// Can't decide between Offset or Zoned variant at runtime if using a T: TimeZone, so
// provide a separate conversion
impl From<(NaiveDateTime, chrono_tz::Tz)> for Value {
    fn from(pair: (NaiveDateTime, chrono_tz::Tz)) -> Self {
        Value::DateTimeZoned(DateTimeZoned::from(pair))
    }
}

impl From<NaiveTime> for Value {
    fn from(value: NaiveTime) -> Self {
        Value::LocalTime(LocalTime::from(value))
    }
}

impl From<NaiveDateTime> for Value {
    fn from(value: NaiveDateTime) -> Self {
        Value::LocalDateTime(LocalDateTime::from(value))
    }
}

impl From<Duration> for Value {
    fn from(value: Duration) -> Self {
        Value::Duration(value)
    }
}

impl From<std::time::Duration> for Value {
    fn from(value: std::time::Duration) -> Self {
        Value::Duration(Duration::from(value))
    }
}

impl From<Point2D> for Value {
    fn from(value: Point2D) -> Self {
        Value::Point2D(value)
    }
}

impl From<Point3D> for Value {
    fn from(value: Point3D) -> Self {
        Value::Point3D(value)
    }
}

// ------------------------- From Value -------------------------

#[doc(hidden)]
macro_rules! impl_try_from_value {
    ($T:path, $V:ident) => {
        impl ::std::convert::TryFrom<$crate::Value> for $T {
            type Error = $crate::error::Error;

            fn try_from(value: $crate::Value) -> $crate::error::Result<Self> {
                match value {
                    $crate::Value::$V(inner) => Ok(inner),
                    _ => Err($crate::error::ConversionError::FromValue(value).into()),
                }
            }
        }
    };
}

impl TryFrom<Value> for bool {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(boolean) => Ok(boolean),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

macro_rules! impl_try_from_value_for_primitives {
    ($($T:ty),+) => {
        $(
            impl TryFrom<crate::Value> for $T {
                type Error = crate::error::Error;

                fn try_from(value: crate::Value) -> crate::error::Result<Self> {
                    match value {
                        // TODO: This could be a lossy cast!
                        crate::Value::Integer(integer) => Ok(integer as $T),
                        _ => Err(crate::error::ConversionError::FromValue(value).into()),
                    }
                }
            }
        )*
    };
}
impl_try_from_value_for_primitives!(i8, i16, i32, i64);

impl TryFrom<Value> for f64 {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Float(float) => Ok(float),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Bytes(byte_array) => Ok(byte_array),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => list.into_iter().map(T::try_from).collect(),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::List(list) => Ok(list),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl<V, S> TryFrom<Value> for HashMap<std::string::String, V, S>
where
    V: TryFrom<Value, Error = Error>,
    S: BuildHasher + Default,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(map) => {
                let mut new_map = HashMap::with_capacity_and_hasher(map.len(), Default::default());
                for (k, v) in map {
                    new_map.insert(k, V::try_from(v)?);
                }
                Ok(new_map)
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl<S> TryFrom<Value> for HashMap<std::string::String, Value, S>
where
    S: BuildHasher + Default,
{
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Map(map) => {
                let mut new_map = HashMap::with_capacity_and_hasher(map.len(), Default::default());
                for (k, v) in map {
                    new_map.insert(k, v);
                }
                Ok(new_map)
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for std::string::String {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::String(string) => Ok(string),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl_try_from_value!(Node, Node);

impl_try_from_value!(Relationship, Relationship);

impl_try_from_value!(Path, Path);

impl_try_from_value!(UnboundRelationship, UnboundRelationship);

impl TryFrom<Value> for NaiveDate {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Date(date) => Ok(date),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for DateTime<FixedOffset> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::DateTimeOffset(date_time_offset) => Ok(FixedOffset::east(
                date_time_offset.offset_seconds,
            )
            .timestamp(
                date_time_offset.epoch_seconds,
                date_time_offset.nanos as u32,
            )),
            Value::DateTimeZoned(date_time_zoned) => {
                // Time zone guaranteed to be valid in existing objects, ok to unwrap
                let timezone: Tz = date_time_zoned.zone_id.parse().unwrap();
                let timezone: FixedOffset = timezone
                    // Get the fixed offset (e.g. Pacific Daylight vs. Pacific Standard)
                    // for the given point in time
                    .offset_from_utc_datetime(
                        &NaiveDateTime::from_timestamp_opt(date_time_zoned.epoch_seconds, 0)
                            // epoch_seconds is guaranteed to be a valid timestamp, ok to
                            // unwrap
                            .unwrap(),
                    )
                    .fix();
                Ok(timezone
                    .timestamp_opt(date_time_zoned.epoch_seconds, date_time_zoned.nanos as u32)
                    // epoch_seconds and nanos are guaranteed to be valid in existing
                    // objects, ok to unwrap
                    .unwrap())
            }
            _ => Err(ConversionError::FromValue(value).into()),
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
                    // epoch_seconds and nanos are guaranteed to be valid in existing
                    // objects, ok to unwrap
                    .unwrap())
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for NaiveTime {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::LocalTime(local_time) => {
                let seconds = (local_time.nanos_since_midnight / 1_000_000_000) as u32;
                let nanos = (local_time.nanos_since_midnight % 1_000_000_000) as u32;
                // We created the LocalTime from a NaiveTime, so it can easily be
                // converted back without worrying about a panic occurring
                Ok(NaiveTime::from_num_seconds_from_midnight(seconds, nanos))
            }
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

impl TryFrom<Value> for NaiveDateTime {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value {
            // We created the LocalDateTime from a NaiveDateTime, so it can easily be
            // converted back without worrying about a panic occurring
            Value::LocalDateTime(local_date_time) => Ok(NaiveDateTime::from_timestamp(
                local_date_time.epoch_seconds,
                local_date_time.nanos as u32,
            )),
            _ => Err(ConversionError::FromValue(value).into()),
        }
    }
}

// We cannot convert to std::time::Duration, since months are not well-defined in terms of
// seconds, and our Duration can hold quantities that are impossible to hold in a
// std::time::Duration (like negative durations).
impl_try_from_value!(Duration, Duration);

impl_try_from_value!(Point2D, Point2D);

impl_try_from_value!(Point3D, Point3D);
