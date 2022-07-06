use std::{
    collections::HashMap,
    mem,
    panic::{catch_unwind, UnwindSafe},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use chrono::{
    DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Timelike,
};
use chrono_tz::Tz;

pub use duration::Duration;
pub use node::Node;
pub use path::Path;
pub use point_2d::Point2D;
pub use point_3d::Point3D;
pub use relationship::Relationship;
pub use unbound_relationship::UnboundRelationship;

use crate::error::*;
use crate::serialization::*;

pub(crate) mod conversions;
pub(crate) mod duration;
pub(crate) mod node;
pub(crate) mod path;
pub(crate) mod point_2d;
pub(crate) mod point_3d;
pub(crate) mod relationship;
pub(crate) mod unbound_relationship;

pub(crate) const MARKER_FALSE: u8 = 0xC2;
pub(crate) const MARKER_TRUE: u8 = 0xC3;
pub(crate) const MARKER_INT_8: u8 = 0xC8;
pub(crate) const MARKER_INT_16: u8 = 0xC9;
pub(crate) const MARKER_INT_32: u8 = 0xCA;
pub(crate) const MARKER_INT_64: u8 = 0xCB;
pub(crate) const MARKER_FLOAT: u8 = 0xC1;
pub(crate) const MARKER_SMALL_BYTES: u8 = 0xCC;
pub(crate) const MARKER_MEDIUM_BYTES: u8 = 0xCD;
pub(crate) const MARKER_LARGE_BYTES: u8 = 0xCE;
pub(crate) const MARKER_TINY_LIST: u8 = 0x90;
pub(crate) const MARKER_SMALL_LIST: u8 = 0xD4;
pub(crate) const MARKER_MEDIUM_LIST: u8 = 0xD5;
pub(crate) const MARKER_LARGE_LIST: u8 = 0xD6;
pub(crate) const MARKER_TINY_MAP: u8 = 0xA0;
pub(crate) const MARKER_SMALL_MAP: u8 = 0xD8;
pub(crate) const MARKER_MEDIUM_MAP: u8 = 0xD9;
pub(crate) const MARKER_LARGE_MAP: u8 = 0xDA;
pub(crate) const MARKER_NULL: u8 = 0xC0;
pub(crate) const MARKER_TINY_STRING: u8 = 0x80;
pub(crate) const MARKER_SMALL_STRING: u8 = 0xD0;
pub(crate) const MARKER_MEDIUM_STRING: u8 = 0xD1;
pub(crate) const MARKER_LARGE_STRING: u8 = 0xD2;
pub(crate) const MARKER_TINY_STRUCT: u8 = 0xB0;
pub(crate) const MARKER_SMALL_STRUCT: u8 = 0xDC;
pub(crate) const MARKER_MEDIUM_STRUCT: u8 = 0xDD;

pub(crate) const SIGNATURE_NODE: u8 = 0x4E;
pub(crate) const SIGNATURE_RELATIONSHIP: u8 = 0x52;
pub(crate) const SIGNATURE_PATH: u8 = 0x50;
pub(crate) const SIGNATURE_UNBOUND_RELATIONSHIP: u8 = 0x72;
pub(crate) const SIGNATURE_DATE: u8 = 0x44;
pub(crate) const SIGNATURE_TIME: u8 = 0x54;
pub(crate) const SIGNATURE_DATE_TIME_OFFSET: u8 = 0x46;
pub(crate) const SIGNATURE_DATE_TIME_ZONED: u8 = 0x66;
pub(crate) const SIGNATURE_LOCAL_TIME: u8 = 0x74;
pub(crate) const SIGNATURE_LOCAL_DATE_TIME: u8 = 0x64;
pub(crate) const SIGNATURE_DURATION: u8 = 0x45;
pub(crate) const SIGNATURE_POINT_2D: u8 = 0x58;
pub(crate) const SIGNATURE_POINT_3D: u8 = 0x59;

/// An enum that can hold values of all Bolt-compatible types.
///
/// Conversions are provided for most types, and are usually pretty intuitive ([`bool`] to
/// [`Value::Boolean`], [`i32`] to [`Value::Integer`], [`HashMap`](std::collections::HashMap) to
/// [`Value::Map`], etc.), but some types have no analog in Rust, like a timezone-aware time. For
/// such types, conversions are still provided, but may feel a bit clunky (for example, you can
/// convert a `(`[`NaiveTime`](chrono::NaiveTime)`, impl `[`Offset`](chrono::Offset)`)` tuple into
/// a [`Value::Time`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // V1-compatible value types
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
    String(String),
    Node(Node),
    Relationship(Relationship),
    Path(Path),
    UnboundRelationship(UnboundRelationship),

    // V2+-compatible value types
    Date(NaiveDate),              // A date without a time zone, i.e. LocalDate
    Time(NaiveTime, FixedOffset), // A time with UTC offset, i.e. OffsetTime
    DateTimeOffset(DateTime<FixedOffset>), // A date-time with UTC offset, i.e. OffsetDateTime
    DateTimeZoned(DateTime<Tz>),  // A date-time with time zone ID, i.e. ZonedDateTime
    LocalTime(NaiveTime),         // A time without time zone
    LocalDateTime(NaiveDateTime), // A date-time without time zone
    Duration(Duration),
    Point2D(Point2D),
    Point3D(Point3D),
}

impl Eq for Value {
    fn assert_receiver_is_total_eq(&self) {
        if let Value::Float(_) | Value::Point2D(_) | Value::Point3D(_) = self {
            panic!("{:?} does not impl Eq", self)
        }
    }
}

impl BoltValue for Value {
    fn marker(&self) -> SerializeResult<u8> {
        match self {
            Value::Boolean(true) => Ok(MARKER_TRUE),
            Value::Boolean(false) => Ok(MARKER_FALSE),
            Value::Integer(integer) => match integer {
                -9_223_372_036_854_775_808..=-2_147_483_649
                | 2_147_483_648..=9_223_372_036_854_775_807 => Ok(MARKER_INT_64),
                -2_147_483_648..=-32_769 | 32_768..=2_147_483_647 => Ok(MARKER_INT_32),
                -32_768..=-129 | 128..=32_767 => Ok(MARKER_INT_16),
                -128..=-17 => Ok(MARKER_INT_8),
                -16..=127 => Ok(*integer as u8),
            },
            Value::Float(_) => Ok(MARKER_FLOAT),
            Value::Bytes(bytes) => match bytes.len() {
                0..=255 => Ok(MARKER_SMALL_BYTES),
                256..=65_535 => Ok(MARKER_MEDIUM_BYTES),
                65_536..=2_147_483_647 => Ok(MARKER_LARGE_BYTES),
                _ => Err(SerializationError::ValueTooLarge(bytes.len())),
            },
            Value::List(list) => match list.len() {
                0..=15 => Ok(MARKER_TINY_LIST | list.len() as u8),
                16..=255 => Ok(MARKER_SMALL_LIST),
                256..=65_535 => Ok(MARKER_MEDIUM_LIST),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_LIST),
                len => Err(SerializationError::ValueTooLarge(len)),
            },
            Value::Map(map) => match map.len() {
                0..=15 => Ok(MARKER_TINY_MAP | map.len() as u8),
                16..=255 => Ok(MARKER_SMALL_MAP),
                256..=65_535 => Ok(MARKER_MEDIUM_MAP),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_MAP),
                _ => Err(SerializationError::ValueTooLarge(map.len())),
            },
            Value::Null => Ok(MARKER_NULL),
            Value::String(string) => match string.len() {
                0..=15 => Ok(MARKER_TINY_STRING | string.len() as u8),
                16..=255 => Ok(MARKER_SMALL_STRING),
                256..=65_535 => Ok(MARKER_MEDIUM_STRING),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_STRING),
                _ => Err(SerializationError::ValueTooLarge(string.len())),
            },
            Value::Node(node) => node.marker(),
            Value::Relationship(rel) => rel.marker(),
            Value::Path(path) => path.marker(),
            Value::UnboundRelationship(unbound_rel) => unbound_rel.marker(),
            Value::Date(_) => Ok(MARKER_TINY_STRUCT | 1),
            Value::Time(_, _) => Ok(MARKER_TINY_STRUCT | 2),
            Value::DateTimeOffset(_) => Ok(MARKER_TINY_STRUCT | 3),
            Value::DateTimeZoned(_) => Ok(MARKER_TINY_STRUCT | 3),
            Value::LocalTime(_) => Ok(MARKER_TINY_STRUCT | 1),
            Value::LocalDateTime(_) => Ok(MARKER_TINY_STRUCT | 2),
            Value::Duration(duration) => duration.marker(),
            Value::Point2D(point_2d) => point_2d.marker(),
            Value::Point3D(point_3d) => point_3d.marker(),
        }
    }

    fn serialize(self) -> SerializeResult<Bytes> {
        let marker = self.marker()?;
        match self {
            Value::Boolean(true) => Ok(Bytes::from_static(&[MARKER_TRUE])),
            Value::Boolean(false) => Ok(Bytes::from_static(&[MARKER_FALSE])),
            Value::Integer(integer) => {
                // Worst case is marker + 64-bit int
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<i64>());

                bytes.put_u8(marker);
                match integer {
                    -9_223_372_036_854_775_808..=-2_147_483_649
                    | 2_147_483_648..=9_223_372_036_854_775_807 => {
                        bytes.put_i64(integer);
                    }
                    -2_147_483_648..=-32_769 | 32_768..=2_147_483_647 => {
                        bytes.put_i32(integer as i32);
                    }
                    -32_768..=-129 | 128..=32_767 => {
                        bytes.put_i16(integer as i16);
                    }
                    -128..=-17 => {
                        bytes.put_i8(integer as i8);
                    }
                    -16..=127 => {} // The marker is the value
                }

                Ok(bytes.freeze())
            }
            Value::Float(f) => {
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<f64>());
                bytes.put_u8(marker);
                bytes.put_f64(f);
                Ok(bytes.freeze())
            }
            Value::Bytes(bytes) => {
                // Worst case is a large ByteArray, with marker byte, 32-bit size value, and length
                let mut buf = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + bytes.len(),
                );

                buf.put_u8(marker);
                match bytes.len() {
                    0..=255 => buf.put_u8(bytes.len() as u8),
                    256..=65_535 => buf.put_u16(bytes.len() as u16),
                    65_536..=2_147_483_647 => buf.put_u32(bytes.len() as u32),
                    _ => return Err(SerializationError::ValueTooLarge(bytes.len())),
                }
                buf.put_slice(&bytes);

                Ok(buf.freeze())
            }
            Value::List(list) => {
                let length = list.len();
                let mut total_value_bytes: usize = 0;
                let mut value_bytes_vec: Vec<Bytes> = Vec::with_capacity(length);

                for value in list {
                    let value_bytes = value.serialize()?;
                    total_value_bytes += value_bytes.len();
                    value_bytes_vec.push(value_bytes);
                }

                // Worst case is a large List, with marker byte, 32-bit size value, and all the
                // Value bytes
                let mut bytes = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + total_value_bytes,
                );

                bytes.put_u8(marker);
                match length {
                    0..=15 => {} // The marker contains the length
                    16..=255 => bytes.put_u8(length as u8),
                    256..=65_535 => bytes.put_u16(length as u16),
                    65_536..=4_294_967_295 => bytes.put_u32(length as u32),
                    _ => return Err(SerializationError::ValueTooLarge(length)),
                }

                for value_bytes in value_bytes_vec {
                    bytes.put(value_bytes);
                }

                Ok(bytes.freeze())
            }
            Value::Map(map) => {
                let length = map.len();

                let mut total_value_bytes: usize = 0;
                let mut value_bytes_vec: Vec<Bytes> = Vec::with_capacity(length);
                for (key, val) in map {
                    let key_bytes: Bytes = Value::String(key).serialize()?;
                    let val_bytes: Bytes = val.serialize()?;
                    total_value_bytes += key_bytes.len() + val_bytes.len();
                    value_bytes_vec.push(key_bytes);
                    value_bytes_vec.push(val_bytes);
                }
                // Worst case is a large Map, with marker byte, 32-bit size value, and all the
                // Value bytes
                let mut bytes = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + total_value_bytes,
                );

                bytes.put_u8(marker);
                match length {
                    0..=15 => {} // The marker contains the length
                    16..=255 => bytes.put_u8(length as u8),
                    256..=65_535 => bytes.put_u16(length as u16),
                    65_536..=4_294_967_295 => bytes.put_u32(length as u32),
                    _ => return Err(SerializationError::ValueTooLarge(length)),
                }

                for value_bytes in value_bytes_vec {
                    bytes.put(value_bytes);
                }

                Ok(bytes.freeze())
            }
            Value::Null => Ok(Bytes::from_static(&[MARKER_NULL])),
            Value::String(string) => {
                let length = string.len();
                // Worst case is a large string, with marker byte, 32-bit size value, and length
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<u32>() + length);

                bytes.put_u8(marker);
                match length {
                    0..=15 => {} // The marker contains the length
                    16..=255 => bytes.put_u8(length as u8),
                    256..=65_535 => bytes.put_u16(length as u16),
                    65_536..=4_294_967_295 => bytes.put_u32(length as u32),
                    _ => return Err(SerializationError::ValueTooLarge(length)),
                }
                bytes.put(string.as_bytes());

                Ok(bytes.freeze())
            }
            Value::Node(node) => node.serialize(),
            Value::Relationship(rel) => rel.serialize(),
            Value::Path(path) => path.serialize(),
            Value::UnboundRelationship(unbound_rel) => unbound_rel.serialize(),
            Value::Date(date) => Ok(vec![marker, SIGNATURE_DATE]
                .into_iter()
                .chain(
                    // Days since UNIX epoch
                    Value::from((date - NaiveDate::from_ymd(1970, 1, 1)).num_days()).serialize()?,
                )
                .collect()),
            Value::Time(time, offset) => Ok(vec![marker, SIGNATURE_TIME]
                .into_iter()
                .chain(
                    // Nanoseconds since midnight
                    // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
                    Value::from(
                        i64::from(time.num_seconds_from_midnight()) * 1_000_000_000
                            + i64::from(time.nanosecond()),
                    )
                    .serialize()?,
                )
                .chain(
                    // Timezone offset
                    Value::from(offset.fix().local_minus_utc()).serialize()?,
                )
                .collect()),
            Value::DateTimeOffset(date_time_offset) => Ok(vec![marker, SIGNATURE_DATE_TIME_OFFSET]
                .into_iter()
                .chain(
                    // Seconds since UNIX epoch
                    Value::from(date_time_offset.timestamp()).serialize()?,
                )
                .chain(
                    // Nanoseconds
                    Value::from(i64::from(date_time_offset.nanosecond())).serialize()?,
                )
                .chain(
                    // Timezone offset
                    Value::from(date_time_offset.offset().fix().local_minus_utc()).serialize()?,
                )
                .collect()),
            Value::DateTimeZoned(date_time_zoned) => {
                Ok(vec![marker, SIGNATURE_DATE_TIME_ZONED]
                    .into_iter()
                    // Seconds since UNIX epoch
                    .chain(Value::from(date_time_zoned.timestamp()).serialize()?)
                    // Nanoseconds
                    .chain(Value::from(i64::from(date_time_zoned.nanosecond())).serialize()?)
                    // Timezone ID
                    .chain(Value::from(date_time_zoned.timezone().name().to_string()).serialize()?)
                    .collect())
            }
            Value::LocalTime(local_time) => Ok(vec![marker, SIGNATURE_LOCAL_TIME]
                .into_iter()
                .chain(
                    Value::from(
                        // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
                        i64::from(local_time.num_seconds_from_midnight()) * 1_000_000_000
                            + i64::from(local_time.nanosecond()),
                    )
                    .serialize()?,
                )
                .collect()),
            Value::LocalDateTime(local_date_time) => Ok(vec![marker, SIGNATURE_LOCAL_DATE_TIME]
                .into_iter()
                // Seconds since UNIX epoch
                .chain(Value::from(local_date_time.timestamp()).serialize()?)
                // Nanoseconds
                .chain(Value::from(i64::from(local_date_time.nanosecond())).serialize()?)
                .collect()),
            Value::Duration(duration) => duration.serialize(),
            Value::Point2D(point_2d) => point_2d.serialize(),
            Value::Point3D(point_3d) => point_3d.serialize(),
        }
    }

    fn deserialize<B: Buf + UnwindSafe>(mut bytes: B) -> DeserializeResult<(Self, B)> {
        catch_unwind(move || {
            let marker = bytes.get_u8();
            match marker {
                // Boolean
                MARKER_TRUE => Ok((Value::Boolean(true), bytes)),
                MARKER_FALSE => Ok((Value::Boolean(false), bytes)),
                // Tiny int
                marker if (-16..=127).contains(&(marker as i8)) => {
                    Ok((Value::Integer(i64::from(marker as i8)), bytes))
                }
                // Other int types
                MARKER_INT_8 => Ok((Value::Integer(i64::from(bytes.get_i8())), bytes)),
                MARKER_INT_16 => Ok((Value::Integer(i64::from(bytes.get_i16())), bytes)),
                MARKER_INT_32 => Ok((Value::Integer(i64::from(bytes.get_i32())), bytes)),
                MARKER_INT_64 => Ok((Value::Integer(bytes.get_i64()), bytes)),
                // Float
                MARKER_FLOAT => Ok((Value::Float(bytes.get_f64()), bytes)),
                // Byte array
                MARKER_SMALL_BYTES | MARKER_MEDIUM_BYTES | MARKER_LARGE_BYTES => {
                    let size = match marker {
                        MARKER_SMALL_BYTES => bytes.get_u8() as usize,
                        MARKER_MEDIUM_BYTES => bytes.get_u16() as usize,
                        MARKER_LARGE_BYTES => bytes.get_u32() as usize,
                        _ => unreachable!(),
                    };
                    Ok((Value::Bytes(bytes.copy_to_bytes(size).to_vec()), bytes))
                }
                // List
                marker
                    if (MARKER_TINY_LIST..=(MARKER_TINY_LIST | 0x0F)).contains(&marker)
                        || matches!(
                            marker,
                            MARKER_SMALL_LIST | MARKER_MEDIUM_LIST | MARKER_LARGE_LIST
                        ) =>
                {
                    let size = match marker {
                        marker
                            if (MARKER_TINY_LIST..=(MARKER_TINY_LIST | 0x0F)).contains(&marker) =>
                        {
                            0x0F & marker as usize
                        }
                        MARKER_SMALL_LIST => bytes.get_u8() as usize,
                        MARKER_MEDIUM_LIST => bytes.get_u16() as usize,
                        MARKER_LARGE_LIST => bytes.get_u32() as usize,
                        _ => unreachable!(),
                    };
                    let mut list: Vec<Value> = Vec::with_capacity(size);
                    for _ in 0..size {
                        let (v, b) = Value::deserialize(bytes)?;
                        bytes = b;
                        list.push(v);
                    }
                    Ok((Value::List(list), bytes))
                }
                // Map
                marker
                    if (MARKER_TINY_MAP..=(MARKER_TINY_MAP | 0x0F)).contains(&marker)
                        || matches!(
                            marker,
                            MARKER_SMALL_MAP | MARKER_MEDIUM_MAP | MARKER_LARGE_MAP
                        ) =>
                {
                    let size = match marker {
                        marker
                            if (MARKER_TINY_MAP..=(MARKER_TINY_MAP | 0x0F)).contains(&marker) =>
                        {
                            0x0F & marker as usize
                        }
                        MARKER_SMALL_MAP => bytes.get_u8() as usize,
                        MARKER_MEDIUM_MAP => bytes.get_u16() as usize,
                        MARKER_LARGE_MAP => bytes.get_u32() as usize,
                        _ => unreachable!(),
                    };

                    let mut hash_map: HashMap<std::string::String, Value> =
                        HashMap::with_capacity(size);
                    for _ in 0..size {
                        let (value, remaining) = Value::deserialize(bytes)?;
                        bytes = remaining;
                        match value {
                            Value::String(key) => {
                                let (value, remaining) = Value::deserialize(bytes)?;
                                bytes = remaining;
                                hash_map.insert(key, value);
                            }
                            other => return Err(ConversionError::FromValue(other).into()),
                        }
                    }

                    Ok((Value::Map(hash_map), bytes))
                }
                // Null
                MARKER_NULL => Ok((Value::Null, bytes)),
                // String
                marker
                    if (MARKER_TINY_STRING..=(MARKER_TINY_STRING | 0x0F)).contains(&marker)
                        || matches!(
                            marker,
                            MARKER_SMALL_STRING | MARKER_MEDIUM_STRING | MARKER_LARGE_STRING
                        ) =>
                {
                    let size = match marker {
                        marker
                            if (MARKER_TINY_STRING..=(MARKER_TINY_STRING | 0x0F))
                                .contains(&marker) =>
                        {
                            0x0F & marker as usize
                        }
                        MARKER_SMALL_STRING => bytes.get_u8() as usize,
                        MARKER_MEDIUM_STRING => bytes.get_u16() as usize,
                        MARKER_LARGE_STRING => bytes.get_u32() as usize,
                        _ => unreachable!(),
                    };

                    Ok((
                        Value::String(String::from_utf8(bytes.copy_to_bytes(size).to_vec())?),
                        bytes,
                    ))
                }
                // Structure
                marker
                    if (MARKER_TINY_STRUCT..=(MARKER_TINY_STRUCT | 0x0F)).contains(&marker)
                        || matches!(marker, MARKER_SMALL_STRUCT | MARKER_MEDIUM_STRUCT) =>
                {
                    deserialize_structure(marker, bytes)
                }
                _ => Err(DeserializationError::InvalidMarkerByte(marker)),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

macro_rules! deserialize_struct {
    ($name:ident, $bytes:ident) => {{
        let (value, remaining) = $name::deserialize($bytes)?;
        $bytes = remaining;
        Ok((Value::$name(value), $bytes))
    }};
}

macro_rules! deserialize_variant {
    ($name:ident, $bytes:ident) => {{
        let (value, remaining) = Value::deserialize($bytes)?;
        $bytes = remaining;
        if let Value::$name(inner) = value {
            inner
        } else {
            return Err(ConversionError::FromValue(value).into());
        }
    }};
}

fn deserialize_structure<B: Buf + UnwindSafe>(
    marker: u8,
    mut bytes: B,
) -> DeserializeResult<(Value, B)> {
    let (_, signature) = get_structure_info(marker, &mut bytes)?;

    match signature {
        SIGNATURE_NODE => deserialize_struct!(Node, bytes),
        SIGNATURE_RELATIONSHIP => deserialize_struct!(Relationship, bytes),
        SIGNATURE_PATH => deserialize_struct!(Path, bytes),
        SIGNATURE_UNBOUND_RELATIONSHIP => deserialize_struct!(UnboundRelationship, bytes),
        SIGNATURE_DATE => {
            let days_since_epoch: i64 = deserialize_variant!(Integer, bytes);
            Ok((
                Value::Date(
                    NaiveDate::from_ymd(1970, 1, 1) + chrono::Duration::days(days_since_epoch),
                ),
                bytes,
            ))
        }
        SIGNATURE_TIME => {
            let nanos_since_midnight: i64 = deserialize_variant!(Integer, bytes);
            let zone_offset: i32 = deserialize_variant!(Integer, bytes) as i32;
            Ok((
                Value::Time(
                    NaiveTime::from_num_seconds_from_midnight(
                        (nanos_since_midnight / 1_000_000_000) as u32,
                        (nanos_since_midnight % 1_000_000_000) as u32,
                    ),
                    FixedOffset::east(zone_offset),
                ),
                bytes,
            ))
        }
        SIGNATURE_DATE_TIME_OFFSET => {
            let epoch_seconds: i64 = deserialize_variant!(Integer, bytes);
            let nanos: i64 = deserialize_variant!(Integer, bytes);
            let offset_seconds: i32 = deserialize_variant!(Integer, bytes) as i32;
            Ok((
                Value::DateTimeOffset(DateTime::from_utc(
                    NaiveDateTime::from_timestamp(epoch_seconds, nanos as u32),
                    FixedOffset::east(offset_seconds),
                )),
                bytes,
            ))
        }
        SIGNATURE_DATE_TIME_ZONED => {
            let epoch_seconds: i64 = deserialize_variant!(Integer, bytes);
            let nanos: i64 = deserialize_variant!(Integer, bytes);
            let timezone_id: String = deserialize_variant!(String, bytes);
            let timezone: Tz = timezone_id.parse().unwrap();
            Ok((
                Value::DateTimeZoned(timezone.timestamp(epoch_seconds, nanos as u32)),
                bytes,
            ))
        }
        SIGNATURE_LOCAL_TIME => {
            let nanos_since_midnight: i64 = deserialize_variant!(Integer, bytes);
            Ok((
                Value::LocalTime(NaiveTime::from_num_seconds_from_midnight(
                    (nanos_since_midnight / 1_000_000_000) as u32,
                    (nanos_since_midnight % 1_000_000_000) as u32,
                )),
                bytes,
            ))
        }
        SIGNATURE_LOCAL_DATE_TIME => {
            let epoch_seconds: i64 = deserialize_variant!(Integer, bytes);
            let nanos: i64 = deserialize_variant!(Integer, bytes);
            Ok((
                Value::LocalDateTime(NaiveDateTime::from_timestamp(epoch_seconds, nanos as u32)),
                bytes,
            ))
        }
        SIGNATURE_DURATION => deserialize_struct!(Duration, bytes),
        SIGNATURE_POINT_2D => deserialize_struct!(Point2D, bytes),
        SIGNATURE_POINT_3D => deserialize_struct!(Point3D, bytes),
        _ => Err(DeserializationError::InvalidSignatureByte(signature)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};

    use super::*;

    macro_rules! value_test {
        ($name:ident, $value:expr, $marker:expr, $($bytes:expr),+) => {
            #[test]
            fn $name() {
                let value = $value;
                let bytes: Bytes = vec![$marker]
                    .into_iter()
                    $(.chain($bytes.iter().copied()))*
                    .collect();
                assert_eq!(value.marker().unwrap(), $marker);
                assert_eq!(value.clone().serialize().unwrap(), &bytes);
                let (deserialized, remaining) = Value::deserialize(bytes).unwrap();
                assert_eq!(deserialized, value);
                assert_eq!(remaining.len(), 0);
            }
        };
        ($name:ident, $value:expr, $marker:expr) => {
            #[test]
            fn $name() {
                let value = $value;
                let bytes = $value.clone().serialize().unwrap();
                assert_eq!(value.marker().unwrap(), $marker);
                let (deserialized, remaining) = Value::deserialize(bytes).unwrap();
                assert_eq!(deserialized, value);
                assert_eq!(remaining.len(), 0);
            }
        };
    }

    value_test!(null, Value::Null, MARKER_NULL, &[]);

    value_test!(bool_true, Value::Boolean(true), MARKER_TRUE, &[]);

    value_test!(bool_false, Value::Boolean(false), MARKER_FALSE, &[]);

    value_test!(tiny_int, Value::Integer(110), 110, &[]);

    value_test!(
        small_int,
        Value::Integer(-127),
        MARKER_INT_8,
        (-127_i8).to_be_bytes()
    );

    value_test!(
        medium_int,
        Value::Integer(8000),
        MARKER_INT_16,
        8000_i16.to_be_bytes()
    );

    value_test!(
        medium_negative_int,
        Value::Integer(-18621),
        MARKER_INT_16,
        (-18621_i16).to_be_bytes()
    );

    value_test!(
        large_int,
        Value::Integer(-1_000_000_000),
        MARKER_INT_32,
        (-1_000_000_000_i32).to_be_bytes()
    );

    value_test!(
        very_large_int,
        Value::Integer(9_000_000_000_000_000_000),
        MARKER_INT_64,
        9_000_000_000_000_000_000_i64.to_be_bytes()
    );

    value_test!(
        float_min,
        Value::Float(f64::MIN_POSITIVE),
        MARKER_FLOAT,
        f64::MIN_POSITIVE.to_be_bytes()
    );

    value_test!(
        float_max,
        Value::Float(f64::MAX),
        MARKER_FLOAT,
        f64::MAX.to_be_bytes()
    );

    value_test!(
        float_e,
        Value::Float(std::f64::consts::E),
        MARKER_FLOAT,
        std::f64::consts::E.to_be_bytes()
    );

    value_test!(
        float_pi,
        Value::Float(std::f64::consts::PI),
        MARKER_FLOAT,
        std::f64::consts::PI.to_be_bytes()
    );

    value_test!(
        empty_byte_array,
        Value::Bytes(vec![]),
        MARKER_SMALL_BYTES,
        &[0]
    );

    value_test!(
        small_byte_array,
        Value::Bytes(vec![1_u8; 100]),
        MARKER_SMALL_BYTES,
        &[100],
        &[1_u8; 100]
    );

    value_test!(
        medium_byte_array,
        Value::Bytes(vec![99_u8; 1000]),
        MARKER_MEDIUM_BYTES,
        1000_u16.to_be_bytes(),
        &[99_u8; 1000]
    );

    value_test!(
        large_byte_array,
        Value::Bytes(vec![1_u8; 100_000]),
        MARKER_LARGE_BYTES,
        100_000_u32.to_be_bytes(),
        &[1_u8; 100_000]
    );

    value_test!(empty_list, Value::List(vec![]), MARKER_TINY_LIST | 0, &[]);

    value_test!(
        tiny_list,
        Value::List(vec![Value::Integer(100_000); 3]),
        MARKER_TINY_LIST | 3,
        &[MARKER_INT_32],
        100_000_u32.to_be_bytes(),
        &[MARKER_INT_32],
        100_000_u32.to_be_bytes(),
        &[MARKER_INT_32],
        100_000_u32.to_be_bytes()
    );

    value_test!(
        small_list,
        Value::List(vec![Value::String(String::from("item")); 100]),
        MARKER_SMALL_LIST,
        &[100],
        &[MARKER_TINY_STRING | 4, b'i', b't', b'e', b'm'].repeat(100)
    );

    value_test!(
        medium_list,
        Value::List(vec![Value::Boolean(false); 1000]),
        MARKER_MEDIUM_LIST,
        1000_u16.to_be_bytes(),
        &[MARKER_FALSE; 1000]
    );

    value_test!(
        large_list,
        Value::List(vec![Value::Integer(1); 70_000]),
        MARKER_LARGE_LIST,
        70_000_u32.to_be_bytes(),
        &[1; 70_000]
    );

    value_test!(
        tiny_string,
        Value::String(String::from("string")),
        MARKER_TINY_STRING | 6,
        b"string"
    );

    value_test!(
        small_string,
        Value::String(String::from("string".repeat(10))),
        MARKER_SMALL_STRING,
        60_u8.to_be_bytes(),
        b"string".repeat(10)
    );

    value_test!(
        medium_string,
        Value::String(String::from("string".repeat(1000))),
        MARKER_MEDIUM_STRING,
        6000_u16.to_be_bytes(),
        b"string".repeat(1000)
    );

    value_test!(
        large_string,
        Value::String(String::from("string".repeat(100_000))),
        MARKER_LARGE_STRING,
        600_000_u32.to_be_bytes(),
        b"string".repeat(100_000)
    );

    value_test!(
        special_string,
        Value::String(String::from("En å flöt över ängen")),
        MARKER_SMALL_STRING,
        24_u8.to_be_bytes(),
        "En å flöt över ängen".bytes().collect::<Vec<_>>()
    );

    value_test!(
        empty_map,
        Value::from(HashMap::<&str, i8>::new()),
        MARKER_TINY_MAP | 0,
        &[]
    );

    value_test!(
        tiny_map,
        Value::from(HashMap::<&str, i8>::from_iter(vec![("a", 1_i8)])),
        MARKER_TINY_MAP | 1,
        &[MARKER_TINY_STRING | 1, b'a', 1]
    );

    #[test]
    fn small_map_from_bytes() {
        let small_map = Value::from(HashMap::<&str, i8>::from_iter(vec![
            ("a", 1_i8),
            ("b", 1_i8),
            ("c", 3_i8),
            ("d", 4_i8),
            ("e", 5_i8),
            ("f", 6_i8),
            ("g", 7_i8),
            ("h", 8_i8),
            ("i", 9_i8),
            ("j", 0_i8),
            ("k", 1_i8),
            ("l", 2_i8),
            ("m", 3_i8),
            ("n", 4_i8),
            ("o", 5_i8),
            ("p", 6_i8),
        ]));
        let bytes = small_map.clone().serialize().unwrap();
        let (deserialized, remaining) = Value::deserialize(bytes).unwrap();
        assert_eq!(deserialized, small_map);
        assert_eq!(remaining.len(), 0);
    }

    value_test!(
        node,
        Value::Node(Node::new(
            24_i64,
            vec!["TestNode".to_string()],
            HashMap::from_iter(vec![
                ("key1".to_string(), -1_i8),
                ("key2".to_string(), 1_i8),
            ]),
        )),
        MARKER_TINY_STRUCT | 3
    );

    value_test!(
        relationship,
        Value::Relationship(Relationship::new(
            24_i64,
            32_i64,
            128_i64,
            "TestRel".to_string(),
            HashMap::from_iter(vec![
                ("key1".to_string(), -2_i8),
                ("key2".to_string(), 2_i8),
            ]),
        )),
        MARKER_TINY_STRUCT | 5
    );

    value_test!(
        path,
        Value::Path(Path::new(
            vec![Node::new(
                24_i64,
                vec!["TestNode".to_string()],
                HashMap::from_iter(vec![
                    ("key1".to_string(), -1_i8),
                    ("key2".to_string(), 1_i8),
                ]),
            )],
            vec![UnboundRelationship::new(
                128_i64,
                "TestRel".to_string(),
                HashMap::from_iter(vec![
                    ("key1".to_string(), -2_i8),
                    ("key2".to_string(), 2_i8),
                ]),
            )],
            vec![100, 101]
        )),
        MARKER_TINY_STRUCT | 3
    );

    value_test!(
        unbound_relationship,
        Value::UnboundRelationship(UnboundRelationship::new(
            128_i64,
            "TestRel".to_string(),
            HashMap::from_iter(vec![
                ("key1".to_string(), -2_i8),
                ("key2".to_string(), 2_i8),
            ]),
        )),
        MARKER_TINY_STRUCT | 3
    );

    value_test!(
        date,
        Value::Date(NaiveDate::from_ymd(2020, 12, 25)),
        MARKER_TINY_STRUCT | 1,
        &[SIGNATURE_DATE],
        &[MARKER_INT_16],
        18621_i16.to_be_bytes()
    );

    value_test!(
        past_date,
        Value::Date(NaiveDate::from_ymd(1901, 12, 31)),
        MARKER_TINY_STRUCT | 1,
        &[SIGNATURE_DATE],
        &[MARKER_INT_16],
        (-24838_i16).to_be_bytes()
    );

    value_test!(
        future_date,
        Value::Date(NaiveDate::from_ymd(3000, 5, 23)),
        MARKER_TINY_STRUCT | 1,
        &[SIGNATURE_DATE],
        &[MARKER_INT_32],
        376342_i32.to_be_bytes()
    );

    value_test!(
        time,
        Value::Time(NaiveTime::from_hms_nano(0, 0, 0, 0), Utc.fix()),
        MARKER_TINY_STRUCT | 2,
        &[SIGNATURE_TIME],
        &[0, 0]
    );

    value_test!(
        about_four_pm_pacific,
        Value::Time(
            NaiveTime::from_hms_nano(16, 4, 35, 235),
            FixedOffset::east(-8 * 3600),
        ),
        MARKER_TINY_STRUCT | 2,
        &[SIGNATURE_TIME],
        &[MARKER_INT_64],
        57875000000235_i64.to_be_bytes(),
        &[MARKER_INT_16],
        (-8 * 3600_i16).to_be_bytes()
    );

    value_test!(
        date_time_offset,
        Value::DateTimeOffset(
            FixedOffset::east(-5 * 3600)
                .from_utc_datetime(&NaiveDate::from_ymd(2050, 12, 31).and_hms_nano(23, 59, 59, 10)),
        ),
        MARKER_TINY_STRUCT | 3,
        &[SIGNATURE_DATE_TIME_OFFSET],
        &[MARKER_INT_64],
        2556143999_i64.to_be_bytes(),
        &[10],
        &[MARKER_INT_16],
        (-5 * 3600_i16).to_be_bytes()
    );

    value_test!(
        date_time_zoned,
        Value::DateTimeZoned(
            chrono_tz::Asia::Ulaanbaatar
                .ymd(2030, 8, 3)
                .and_hms_milli(14, 30, 1, 2),
        ),
        MARKER_TINY_STRUCT | 3,
        &[SIGNATURE_DATE_TIME_ZONED],
        &[MARKER_INT_32],
        1911969001_i32.to_be_bytes(),
        &[MARKER_INT_32],
        2000000_i32.to_be_bytes(),
        &[MARKER_SMALL_STRING, 16],
        b"Asia/Ulaanbaatar"
    );

    value_test!(
        local_time,
        Value::LocalTime(NaiveTime::from_hms_nano(23, 59, 59, 999)),
        MARKER_TINY_STRUCT | 1,
        &[SIGNATURE_LOCAL_TIME],
        &[MARKER_INT_64],
        86399000000999_i64.to_be_bytes()
    );

    value_test!(
        local_date_time,
        Value::LocalDateTime(NaiveDate::from_ymd(1999, 2, 27).and_hms_nano(1, 0, 0, 9999)),
        MARKER_TINY_STRUCT | 2,
        &[SIGNATURE_LOCAL_DATE_TIME],
        &[MARKER_INT_32],
        920077200_i32.to_be_bytes(),
        &[MARKER_INT_16],
        9999_i16.to_be_bytes()
    );

    value_test!(
        duration,
        Value::Duration(Duration::new(9876, 12345, 65332, 23435)),
        MARKER_TINY_STRUCT | 4,
        &[SIGNATURE_DURATION],
        &[MARKER_INT_16],
        9876_i16.to_be_bytes(),
        &[MARKER_INT_16],
        12345_i16.to_be_bytes(),
        &[MARKER_INT_32],
        65332_i32.to_be_bytes(),
        &[MARKER_INT_16],
        23435_i16.to_be_bytes()
    );

    value_test!(
        point_2d,
        Value::Point2D(Point2D::new(9876, 12.312_345, 134_564.123_567_543)),
        MARKER_TINY_STRUCT | 3,
        &[SIGNATURE_POINT_2D],
        &[MARKER_INT_16],
        9876_i16.to_be_bytes(),
        &[MARKER_FLOAT],
        12.312345_f64.to_be_bytes(),
        &[MARKER_FLOAT],
        134_564.123_567_543_f64.to_be_bytes()
    );

    value_test!(
        point_3d,
        Value::Point3D(Point3D::new(
            249,
            543.598_387,
            2_945_732_849.293_85,
            45_438.874_385
        )),
        MARKER_TINY_STRUCT | 4,
        &[SIGNATURE_POINT_3D],
        &[MARKER_INT_16],
        249_i16.to_be_bytes(),
        &[MARKER_FLOAT],
        543.598_387_f64.to_be_bytes(),
        &[MARKER_FLOAT],
        2_945_732_849.293_85_f64.to_be_bytes(),
        &[MARKER_FLOAT],
        45_438.874_385_f64.to_be_bytes()
    );

    #[test]
    #[ignore]
    fn value_size() {
        use std::mem::size_of;
        println!("Duration: {} bytes", size_of::<Duration>());
        println!("Node: {} bytes", size_of::<Node>());
        println!("Path: {} bytes", size_of::<Path>());
        println!("Point2D: {} bytes", size_of::<Point2D>());
        println!("Point3D: {} bytes", size_of::<Point3D>());
        println!("Relationship: {} bytes", size_of::<Relationship>());
        println!(
            "UnboundRelationship: {} bytes",
            size_of::<UnboundRelationship>()
        );
        println!("Value: {} bytes", size_of::<Value>())
    }
}
