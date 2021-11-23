use std::{collections::HashMap, hash::BuildHasher};

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone};
use chrono_tz::Tz;

use crate::value::*;

// ------------------------- Into Value -------------------------

#[doc(hidden)]
macro_rules! impl_from {
    ($T:ty, $V:ident) => {
        impl ::std::convert::From<$T> for $crate::Value {
            fn from(value: $T) -> Self {
                Value::$V(value)
            }
        }
    };
}

impl_from!(bool, Boolean);

macro_rules! impl_from_int {
    ($($T:ty),+) => {
        $(
            impl ::std::convert::From<$T> for $crate::Value {
                fn from(value: $T) -> Self {
                    Value::Integer(value as i64)
                }
            }
        )*
    };
}
impl_from_int!(i8, i16, i32, i64);

impl_from!(f64, Float);

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Value::Bytes(value.to_vec())
    }
}

impl_from!(Vec<u8>, Bytes);

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

impl_from!(String, String);

impl_from!(Node, Node);

impl_from!(Relationship, Relationship);

impl_from!(Path, Path);

impl_from!(UnboundRelationship, UnboundRelationship);

impl_from!(NaiveDate, Date);

// No timezone-aware time in chrono, so provide a separate conversion
impl<O: Offset> From<(NaiveTime, O)> for Value {
    fn from(pair: (NaiveTime, O)) -> Self {
        Value::Time(pair.0, pair.1.fix())
    }
}

impl<T: TimeZone> From<DateTime<T>> for Value {
    fn from(value: DateTime<T>) -> Self {
        Value::DateTimeOffset(DateTime::from_utc(value.naive_utc(), value.offset().fix()))
    }
}

// Can't decide between Offset or Zoned variant at runtime if using a T: TimeZone, so
// provide a separate conversion
impl From<(NaiveDateTime, chrono_tz::Tz)> for Value {
    fn from(pair: (NaiveDateTime, chrono_tz::Tz)) -> Self {
        Value::DateTimeZoned(pair.1.from_utc_datetime(&pair.0))
    }
}

impl_from!(NaiveTime, LocalTime);

impl_from!(NaiveDateTime, LocalDateTime);

impl_from!(Duration, Duration);

impl From<std::time::Duration> for Value {
    fn from(value: std::time::Duration) -> Self {
        Value::Duration(Duration::from(value))
    }
}

impl_from!(Point2D, Point2D);

impl_from!(Point3D, Point3D);

// ------------------------- From Value -------------------------

#[doc(hidden)]
macro_rules! impl_try_from_value {
    ($T:ty, $V:ident) => {
        impl ::std::convert::TryFrom<$crate::Value> for $T {
            type Error = $crate::error::ConversionError;

            fn try_from(value: $crate::Value) -> $crate::error::ConversionResult<Self> {
                match value {
                    $crate::Value::$V(inner) => Ok(inner),
                    _ => Err($crate::error::ConversionError::FromValue(value)),
                }
            }
        }
    };
}

impl_try_from_value!(bool, Boolean);

macro_rules! impl_try_from_value_for_ints {
    ($($T:ty),+) => {
        $(
            impl TryFrom<$crate::Value> for $T {
                type Error = $crate::error::ConversionError;

                fn try_from(value: $crate::Value) -> $crate::error::ConversionResult<Self> {
                    use ::std::convert::TryInto;

                    match value {
                        $crate::Value::Integer(integer) => Ok(integer.try_into()?),
                        _ => Err($crate::error::ConversionError::FromValue(value)),
                    }
                }
            }
        )*
    };
}
impl_try_from_value_for_ints!(i8, i16, i32, i64);

impl_try_from_value!(f64, Float);

impl_try_from_value!(Vec<u8>, Bytes);

impl<T> TryFrom<Value> for Vec<T>
where
    T: TryFrom<Value, Error = ConversionError>,
{
    type Error = ConversionError;

    fn try_from(value: Value) -> ConversionResult<Self> {
        match value {
            Value::List(list) => list.into_iter().map(T::try_from).collect(),
            _ => Err(ConversionError::FromValue(value)),
        }
    }
}

impl_try_from_value!(Vec<Value>, List);

impl<V, S> TryFrom<Value> for HashMap<std::string::String, V, S>
where
    V: TryFrom<Value, Error = ConversionError>,
    S: BuildHasher + Default,
{
    type Error = ConversionError;

    fn try_from(value: Value) -> ConversionResult<Self> {
        match value {
            Value::Map(map) => {
                let mut new_map = HashMap::with_capacity_and_hasher(map.len(), Default::default());
                for (k, v) in map {
                    new_map.insert(k, V::try_from(v)?);
                }
                Ok(new_map)
            }
            _ => Err(ConversionError::FromValue(value)),
        }
    }
}

impl<S> TryFrom<Value> for HashMap<std::string::String, Value, S>
where
    S: BuildHasher + Default,
{
    type Error = ConversionError;

    fn try_from(value: Value) -> ConversionResult<Self> {
        match value {
            Value::Map(map) => {
                let mut new_map = HashMap::with_capacity_and_hasher(map.len(), Default::default());
                for (k, v) in map {
                    new_map.insert(k, v);
                }
                Ok(new_map)
            }
            _ => Err(ConversionError::FromValue(value)),
        }
    }
}

impl_try_from_value!(String, String);

impl_try_from_value!(Node, Node);

impl_try_from_value!(Relationship, Relationship);

impl_try_from_value!(Path, Path);

impl_try_from_value!(UnboundRelationship, UnboundRelationship);

impl_try_from_value!(NaiveDate, Date);

impl TryFrom<Value> for DateTime<FixedOffset> {
    type Error = ConversionError;

    fn try_from(value: Value) -> ConversionResult<Self> {
        match value {
            Value::DateTimeOffset(date_time_offset) => Ok(date_time_offset),
            Value::DateTimeZoned(date_time_zoned) => {
                Ok(date_time_zoned.with_timezone(&date_time_zoned.offset().fix()))
            }
            _ => Err(ConversionError::FromValue(value)),
        }
    }
}

impl_try_from_value!(DateTime<Tz>, DateTimeZoned);

impl_try_from_value!(NaiveTime, LocalTime);

impl_try_from_value!(NaiveDateTime, LocalDateTime);

// We cannot convert to std::time::Duration, since months are not well-defined in terms of
// seconds, and our Duration can hold quantities that are impossible to hold in a
// std::time::Duration (like negative durations).
impl_try_from_value!(Duration, Duration);

impl_try_from_value!(Point2D, Point2D);

impl_try_from_value!(Point3D, Point3D);
