use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    mem,
    ops::DerefMut,
    panic::{catch_unwind, UnwindSafe},
    sync::{Arc, Mutex},
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

pub(crate) const SIGNATURE_DATE: u8 = 0x44;
pub(crate) const SIGNATURE_TIME: u8 = 0x54;
pub(crate) const SIGNATURE_DATE_TIME_OFFSET: u8 = 0x46;
pub(crate) const SIGNATURE_DATE_TIME_ZONED: u8 = 0x66;
pub(crate) const SIGNATURE_LOCAL_TIME: u8 = 0x74;
pub(crate) const SIGNATURE_LOCAL_DATE_TIME: u8 = 0x64;

/// An enum that can hold values of all Bolt-compatible types.
///
/// Conversions are provided for most types, and are usually pretty intuitive ([`bool`] to
/// [`Value::Boolean`], [`i32`] to [`Value::Integer`],
/// [`HashMap`](std::collections::HashMap) to [`Value::Map`], etc.), but some types have
/// no analog in Rust, like a timezone-aware time. For such types, conversions are still
/// provided, but may feel a bit clunky (for example, you can convert a
/// `(`[`NaiveTime`](chrono::NaiveTime)`, impl `[`Offset`](chrono::Offset)`)` tuple into a
/// [`Value::Time`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // V1-compatible value types
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(HashMap<std::string::String, Value>),
    Null,
    String(String),
    Node(Node),
    Relationship(Relationship),
    Path(Path),
    UnboundRelationship(UnboundRelationship),

    // V2+-compatible value types
    Date(NaiveDate),              // A date without a time zone, a.k.a. LocalDate
    Time(NaiveTime, FixedOffset), // A time with a UTC offset, a.k.a. OffsetTime
    DateTimeOffset(DateTime<FixedOffset>), // A date-time with a UTC offset, a.k.a. OffsetDateTime
    DateTimeZoned(DateTime<Tz>),  // A date-time with a time zone ID, a.k.a. ZonedDateTime
    LocalTime(NaiveTime),         // A time without a time zone
    LocalDateTime(NaiveDateTime), // A date-time without a time zone
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

// TODO: This can be implemented for foreign types and delegated to those impls
impl BoltValue for Value {
    fn marker(&self) -> MarkerResult<u8> {
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
                _ => Err(MarkerError::ValueTooLarge(bytes.len())),
            },
            Value::List(list) => match list.len() {
                0..=15 => Ok(MARKER_TINY_LIST | list.len() as u8),
                16..=255 => Ok(MARKER_SMALL_LIST),
                256..=65_535 => Ok(MARKER_MEDIUM_LIST),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_LIST),
                len => Err(MarkerError::ValueTooLarge(len)),
            },
            Value::Map(map) => match map.len() {
                0..=15 => Ok(MARKER_TINY_MAP | map.len() as u8),
                16..=255 => Ok(MARKER_SMALL_MAP),
                256..=65_535 => Ok(MARKER_MEDIUM_MAP),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_MAP),
                _ => Err(MarkerError::ValueTooLarge(map.len())),
            },
            Value::Null => Ok(MARKER_NULL),
            Value::String(string) => match string.len() {
                0..=15 => Ok(MARKER_TINY_STRING | string.len() as u8),
                16..=255 => Ok(MARKER_SMALL_STRING),
                256..=65_535 => Ok(MARKER_MEDIUM_STRING),
                65_536..=4_294_967_295 => Ok(MARKER_LARGE_STRING),
                _ => Err(MarkerError::ValueTooLarge(string.len())),
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
                    _ => return Err(MarkerError::ValueTooLarge(bytes.len()).into()),
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
                    _ => return Err(MarkerError::ValueTooLarge(length).into()),
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
                    _ => return Err(MarkerError::ValueTooLarge(length).into()),
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
                    _ => return Err(MarkerError::ValueTooLarge(length).into()),
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
                        time.num_seconds_from_midnight() as i64 * 1_000_000_000
                            + time.nanosecond() as i64,
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
                    Value::from(date_time_offset.nanosecond() as i64).serialize()?,
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
                    .chain(Value::from(date_time_zoned.nanosecond() as i64).serialize()?)
                    // Timezone ID
                    .chain(Value::from(date_time_zoned.timezone().name().to_string()).serialize()?)
                    .collect())
            }
            Value::LocalTime(local_time) => Ok(vec![marker, SIGNATURE_LOCAL_TIME]
                .into_iter()
                .chain(
                    Value::from(
                        // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
                        local_time.num_seconds_from_midnight() as i64 * 1_000_000_000
                            + local_time.nanosecond() as i64,
                    )
                    .serialize()?,
                )
                .collect()),
            Value::LocalDateTime(local_date_time) => Ok(vec![marker, SIGNATURE_LOCAL_DATE_TIME]
                .into_iter()
                // Seconds since UNIX epoch
                .chain(Value::from(local_date_time.timestamp()).serialize()?)
                // Nanoseconds
                .chain(Value::from(local_date_time.nanosecond() as i64).serialize()?)
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
                    Ok((Value::Integer(marker as i8 as i64), bytes))
                }
                // Other int types
                MARKER_INT_8 => Ok((Value::Integer(bytes.get_i8() as i64), bytes)),
                MARKER_INT_16 => Ok((Value::Integer(bytes.get_i16() as i64), bytes)),
                MARKER_INT_32 => Ok((Value::Integer(bytes.get_i32() as i64), bytes)),
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
                    deserialize_structure_new(bytes)
                }
                _ => Err(DeserializationError::InvalidMarkerByte(marker)),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

macro_rules! deserialize {
    ($name:ident, $bytes:ident) => {{
        let (value, remaining) = $name::deserialize($bytes)?;
        $bytes = remaining;
        Ok((Value::$name(value), $bytes))
    }};
}

fn deserialize_structure_new<B: Buf + UnwindSafe>(mut bytes: B) -> DeserializeResult<(Value, B)> {
    let (_, _, signature) = get_structure_info(&mut bytes)?;
    match signature {
        node::SIGNATURE => deserialize!(Node, bytes),
        relationship::SIGNATURE => deserialize!(Relationship, bytes),
        path::SIGNATURE => deserialize!(Path, bytes),
        unbound_relationship::SIGNATURE => deserialize!(UnboundRelationship, bytes),
        // TODO
        // SIGNATURE_DATE => {
        //     let days_since_epoch: i64 = Value::deserialize(bytes)?.try_into()?;
        //     Ok(Value::Date(
        //         NaiveDate::from_ymd(1970, 1, 1) + chrono::Duration::days(days_since_epoch),
        //     ))
        // }
        // SIGNATURE_TIME => {
        //     let nanos_since_midnight: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let zone_offset: i32 = Value::try_from(input_arc)?.try_into()?;
        //     Ok(Value::Time(
        //         NaiveTime::from_num_seconds_from_midnight(
        //             (nanos_since_midnight / 1_000_000_000) as u32,
        //             (nanos_since_midnight % 1_000_000_000) as u32,
        //         ),
        //         FixedOffset::east(zone_offset),
        //     ))
        // }
        // SIGNATURE_DATE_TIME_OFFSET => {
        //     let epoch_seconds: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let nanos: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let offset_seconds: i32 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     Ok(Value::DateTimeOffset(DateTime::from_utc(
        //         NaiveDateTime::from_timestamp(epoch_seconds, nanos as u32),
        //         FixedOffset::east(offset_seconds),
        //     )))
        // }
        // SIGNATURE_DATE_TIME_ZONED => {
        //     let epoch_seconds: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let nanos: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let timezone_id: String = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let timezone: Tz = timezone_id.parse().unwrap();
        //     Ok(Value::DateTimeZoned(
        //         timezone.timestamp(epoch_seconds, nanos as u32),
        //     ))
        // }
        // SIGNATURE_LOCAL_TIME => {
        //     let nanos_since_midnight: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     Ok(Value::LocalTime(NaiveTime::from_num_seconds_from_midnight(
        //         (nanos_since_midnight / 1_000_000_000) as u32,
        //         (nanos_since_midnight % 1_000_000_000) as u32,
        //     )))
        // }
        // SIGNATURE_LOCAL_DATE_TIME => {
        //     let epoch_seconds: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     let nanos: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
        //     Ok(Value::LocalDateTime(NaiveDateTime::from_timestamp(
        //         epoch_seconds,
        //         nanos as u32,
        //     )))
        // }
        duration::SIGNATURE => deserialize!(Duration, bytes),
        point_2d::SIGNATURE => deserialize!(Point2D, bytes),
        point_3d::SIGNATURE => deserialize!(Point3D, bytes),
        _ => Err(DeserializationError::InvalidSignatureByte(signature)),
    }
}

impl Marker for Value {
    fn get_marker(&self) -> Result<u8> {
        Ok(self.marker()?)
    }
}

impl Serialize for Value {}

impl TryInto<Bytes> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        match self {
            Value::Boolean(true) => Ok(Bytes::from_static(&[MARKER_TRUE])),
            Value::Boolean(false) => Ok(Bytes::from_static(&[MARKER_FALSE])),
            Value::Integer(integer) => {
                // Worst case is 64-bit int
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<i64>());

                match integer {
                    -9_223_372_036_854_775_808..=-2_147_483_649
                    | 2_147_483_648..=9_223_372_036_854_775_807 => {
                        bytes.put_u8(MARKER_INT_64);
                        bytes.put_i64(integer);
                    }
                    -2_147_483_648..=-32_769 | 32_768..=2_147_483_647 => {
                        bytes.put_u8(MARKER_INT_32);
                        bytes.put_i32(integer as i32);
                    }
                    -32_768..=-129 | 128..=32_767 => {
                        bytes.put_u8(MARKER_INT_16);
                        bytes.put_i16(integer as i16);
                    }
                    -128..=-17 => {
                        bytes.put_u8(MARKER_INT_8);
                        bytes.put_i8(integer as i8);
                    }
                    -16..=127 => bytes.put_u8(integer as u8),
                }

                Ok(bytes.freeze())
            }
            Value::Float(f) => {
                let mut bytes =
                    BytesMut::with_capacity(mem::size_of::<u8>() + mem::size_of::<f64>());
                bytes.put_u8(MARKER_FLOAT);
                bytes.put_f64(f);
                Ok(bytes.freeze())
            }
            Value::Bytes(bytes) => {
                // Worst case is a large ByteArray, with marker byte, 32-bit size value, and length
                let mut buf = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + bytes.len(),
                );
                match bytes.len() {
                    0..=255 => {
                        buf.put_u8(MARKER_SMALL_BYTES);
                        buf.put_u8(bytes.len() as u8)
                    }
                    256..=65_535 => {
                        buf.put_u8(MARKER_MEDIUM_BYTES);
                        buf.put_u16(bytes.len() as u16)
                    }
                    65_536..=2_147_483_647 => {
                        buf.put_u8(MARKER_LARGE_BYTES);
                        buf.put_u32(bytes.len() as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(bytes.len())),
                }
                buf.put_slice(&bytes);
                Ok(buf.freeze())
            }
            Value::List(list) => {
                let length = list.len();

                let mut total_value_bytes: usize = 0;
                let mut value_bytes_vec: Vec<Bytes> = Vec::with_capacity(length);
                for value in list {
                    let value_bytes: Bytes = value.try_into()?;
                    total_value_bytes += value_bytes.len();
                    value_bytes_vec.push(value_bytes);
                }

                // Worst case is a large List, with marker byte, 32-bit size value, and all the
                // Value bytes
                let mut bytes = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + total_value_bytes,
                );

                match length {
                    0..=15 => bytes.put_u8(MARKER_TINY_LIST | length as u8),
                    16..=255 => {
                        bytes.put_u8(MARKER_SMALL_LIST);
                        bytes.put_u8(length as u8)
                    }
                    256..=65_535 => {
                        bytes.put_u8(MARKER_MEDIUM_LIST);
                        bytes.put_u16(length as u16);
                    }
                    65_536..=4_294_967_295 => {
                        bytes.put_u8(MARKER_LARGE_LIST);
                        bytes.put_u32(length as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(length)),
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
                    let key_bytes: Bytes = Value::String(key).try_into()?;
                    let val_bytes: Bytes = val.try_into()?;
                    total_value_bytes += key_bytes.len() + val_bytes.len();
                    value_bytes_vec.push(key_bytes);
                    value_bytes_vec.push(val_bytes);
                }
                // Worst case is a large Map, with marker byte, 32-bit size value, and all the
                // Value bytes
                let mut bytes = BytesMut::with_capacity(
                    mem::size_of::<u8>() + mem::size_of::<u32>() + total_value_bytes,
                );

                match length {
                    0..=15 => bytes.put_u8(MARKER_TINY_MAP | length as u8),
                    16..=255 => {
                        bytes.put_u8(MARKER_SMALL_MAP);
                        bytes.put_u8(length as u8)
                    }
                    256..=65_535 => {
                        bytes.put_u8(MARKER_MEDIUM_MAP);
                        bytes.put_u16(length as u16);
                    }
                    65_536..=4_294_967_295 => {
                        bytes.put_u8(MARKER_LARGE_MAP);
                        bytes.put_u32(length as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(length)),
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

                match length {
                    0..=15 => bytes.put_u8(MARKER_TINY_STRING | length as u8),
                    16..=255 => {
                        bytes.put_u8(MARKER_SMALL_STRING);
                        bytes.put_u8(length as u8)
                    }
                    256..=65_535 => {
                        bytes.put_u8(MARKER_MEDIUM_STRING);
                        bytes.put_u16(length as u16);
                    }
                    65_536..=4_294_967_295 => {
                        bytes.put_u8(MARKER_LARGE_STRING);
                        bytes.put_u32(length as u32)
                    }
                    _ => return Err(Error::ValueTooLarge(length)),
                }

                bytes.put_slice(string.as_bytes());
                Ok(bytes.freeze())
            }
            Value::Node(node) => node.try_into(),
            Value::Relationship(rel) => rel.try_into(),
            Value::Path(path) => path.try_into(),
            Value::UnboundRelationship(unbound_rel) => unbound_rel.try_into(),
            Value::Date(date) => Ok(Bytes::from_iter(
                vec![MARKER_TINY_STRUCT | 1, SIGNATURE_DATE]
                    .into_iter()
                    .chain(
                        // Days since UNIX epoch
                        Value::from((date - NaiveDate::from_ymd(1970, 1, 1)).num_days())
                            .try_into_bytes()?,
                    ),
            )),
            Value::Time(time, offset) => Ok(Bytes::from_iter(
                vec![MARKER_TINY_STRUCT | 2, SIGNATURE_TIME]
                    .into_iter()
                    .chain(
                        // Nanoseconds since midnight
                        // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
                        Value::from(
                            time.num_seconds_from_midnight() as i64 * 1_000_000_000
                                + time.nanosecond() as i64,
                        )
                        .try_into_bytes()?,
                    )
                    .chain(
                        // Timezone offset
                        Value::from(offset.fix().local_minus_utc()).try_into_bytes()?,
                    ),
            )),
            Value::DateTimeOffset(date_time_offset) => Ok(Bytes::from_iter(
                vec![MARKER_TINY_STRUCT | 3, SIGNATURE_DATE_TIME_OFFSET]
                    .into_iter()
                    .chain(
                        // Seconds since UNIX epoch
                        Value::from(date_time_offset.timestamp()).try_into_bytes()?,
                    )
                    .chain(
                        // Nanoseconds
                        Value::from(date_time_offset.nanosecond() as i64).try_into_bytes()?,
                    )
                    .chain(
                        // Timezone offset
                        Value::from(date_time_offset.offset().fix().local_minus_utc())
                            .try_into_bytes()?,
                    ),
            )),
            Value::DateTimeZoned(date_time_zoned) => {
                Ok(Bytes::from_iter(
                    vec![MARKER_TINY_STRUCT | 3, SIGNATURE_DATE_TIME_ZONED]
                        .into_iter()
                        // Seconds since UNIX epoch
                        .chain(Value::from(date_time_zoned.timestamp()).try_into_bytes()?)
                        // Nanoseconds
                        .chain(Value::from(date_time_zoned.nanosecond() as i64).try_into_bytes()?)
                        // Timezone ID
                        .chain(
                            Value::from(date_time_zoned.timezone().name().to_string())
                                .try_into_bytes()?,
                        ),
                ))
            }
            Value::LocalTime(local_time) => Ok(Bytes::from_iter(
                vec![MARKER_TINY_STRUCT | 1, SIGNATURE_LOCAL_TIME]
                    .into_iter()
                    .chain(
                        Value::from(
                            // Will not overflow: u32::MAX * 1_000_000_000 + u32::MAX < i64::MAX
                            local_time.num_seconds_from_midnight() as i64 * 1_000_000_000
                                + local_time.nanosecond() as i64,
                        )
                        .try_into_bytes()?,
                    ),
            )),
            Value::LocalDateTime(local_date_time) => Ok(Bytes::from_iter(
                vec![MARKER_TINY_STRUCT | 2, SIGNATURE_LOCAL_DATE_TIME]
                    .into_iter()
                    // Seconds since UNIX epoch
                    .chain(Value::from(local_date_time.timestamp()).try_into_bytes()?)
                    // Nanoseconds
                    .chain(Value::from(local_date_time.nanosecond() as i64).try_into_bytes()?),
            )),
            Value::Duration(duration) => duration.try_into(),
            Value::Point2D(point_2d) => point_2d.try_into(),
            Value::Point3D(point_3d) => point_3d.try_into(),
        }
    }
}

impl Deserialize for Value {}

impl TryFrom<Arc<Mutex<Bytes>>> for Value {
    type Error = Error;

    fn try_from(input_arc: Arc<Mutex<Bytes>>) -> Result<Self> {
        catch_unwind(move || {
            let marker = input_arc.lock().unwrap()[0];
            match marker {
                MARKER_FALSE => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Boolean(false))
                }
                MARKER_TRUE => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Boolean(true))
                }
                // Tiny int
                marker if (-16..=127).contains(&(marker as i8)) => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Integer(marker as i8 as i64))
                }
                // Other int types
                MARKER_INT_8 | MARKER_INT_16 | MARKER_INT_32 | MARKER_INT_64 => {
                    let mut input_bytes = input_arc.lock().unwrap();
                    let marker = input_bytes.get_u8();

                    match marker {
                        MARKER_INT_8 => Ok(Value::Integer(input_bytes.get_i8() as i64)),
                        MARKER_INT_16 => Ok(Value::Integer(input_bytes.get_i16() as i64)),
                        MARKER_INT_32 => Ok(Value::Integer(input_bytes.get_i32() as i64)),
                        MARKER_INT_64 => Ok(Value::Integer(input_bytes.get_i64() as i64)),
                        _ => Err(DeserializationError::InvalidMarkerByte(marker).into()),
                    }
                }
                MARKER_FLOAT => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Float(input_arc.lock().unwrap().get_f64()))
                }
                MARKER_SMALL_BYTES | MARKER_MEDIUM_BYTES | MARKER_LARGE_BYTES => {
                    let mut input_bytes = input_arc.lock().unwrap();
                    let marker = input_bytes.get_u8();
                    let size = match marker {
                        MARKER_SMALL_BYTES => input_bytes.get_u8() as usize,
                        MARKER_MEDIUM_BYTES => input_bytes.get_u16() as usize,
                        MARKER_LARGE_BYTES => input_bytes.get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };
                    let mut bytes = vec![0; size];
                    input_bytes.copy_to_slice(&mut bytes);
                    Ok(Value::Bytes(bytes))
                }
                // Tiny list
                marker if (MARKER_TINY_LIST..=(MARKER_TINY_LIST | 0x0F)).contains(&marker) => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = 0x0F & marker as usize;
                    let mut list: Vec<Value> = Vec::with_capacity(size);
                    for _ in 0..size {
                        list.push(Value::try_from(Arc::clone(&input_arc))?);
                    }
                    Ok(Value::List(list))
                }
                MARKER_SMALL_LIST | MARKER_MEDIUM_LIST | MARKER_LARGE_LIST => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = match marker {
                        MARKER_SMALL_LIST => input_arc.lock().unwrap().get_u8() as usize,
                        MARKER_MEDIUM_LIST => input_arc.lock().unwrap().get_u16() as usize,
                        MARKER_LARGE_LIST => input_arc.lock().unwrap().get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };
                    let mut list: Vec<Value> = Vec::with_capacity(size);
                    for _ in 0..size {
                        list.push(Value::try_from(Arc::clone(&input_arc))?);
                    }
                    Ok(Value::List(list))
                }
                // Tiny map
                marker if (MARKER_TINY_MAP..=(MARKER_TINY_MAP | 0x0F)).contains(&marker) => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = 0x0F & marker as usize;
                    let mut hash_map: HashMap<std::string::String, Value> =
                        HashMap::with_capacity(size);
                    for _ in 0..size {
                        let key = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
                        let value = Value::try_from(Arc::clone(&input_arc))?;
                        hash_map.insert(key, value);
                    }

                    Ok(Value::Map(hash_map))
                }
                MARKER_SMALL_MAP | MARKER_MEDIUM_MAP | MARKER_LARGE_MAP => {
                    let marker = input_arc.lock().unwrap().get_u8();
                    let size = match marker {
                        MARKER_SMALL_MAP => input_arc.lock().unwrap().get_u8() as usize,
                        MARKER_MEDIUM_MAP => input_arc.lock().unwrap().get_u16() as usize,
                        MARKER_LARGE_MAP => input_arc.lock().unwrap().get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };

                    let mut hash_map: HashMap<std::string::String, Value> =
                        HashMap::with_capacity(size);
                    for _ in 0..size {
                        let key = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
                        let value = Value::try_from(Arc::clone(&input_arc))?;
                        hash_map.insert(key, value);
                    }

                    Ok(Value::Map(hash_map))
                }
                MARKER_NULL => {
                    input_arc.lock().unwrap().advance(1);
                    Ok(Value::Null)
                }
                // Tiny string
                marker if (MARKER_TINY_STRING..=(MARKER_TINY_STRING | 0x0F)).contains(&marker) => {
                    let mut input_bytes = input_arc.lock().unwrap();
                    let marker = input_bytes.get_u8();
                    // Lower-order nibble of tiny string marker
                    let size = 0x0F & marker as usize;

                    let mut string_bytes = vec![0; size];
                    // We resize here so that the length of string_bytes is nonzero, which allows
                    // us to use copy_to_slice
                    input_bytes.copy_to_slice(&mut string_bytes);
                    Ok(Value::String(
                        std::string::String::from_utf8(string_bytes.to_vec())
                            .map_err(DeserializationError::InvalidUTF8)?,
                    ))
                }
                MARKER_SMALL_STRING | MARKER_MEDIUM_STRING | MARKER_LARGE_STRING => {
                    let mut input_bytes = input_arc.lock().unwrap();
                    let marker = input_bytes.get_u8();
                    let size = match marker {
                        MARKER_SMALL_STRING => input_bytes.get_u8() as usize,
                        MARKER_MEDIUM_STRING => input_bytes.get_u16() as usize,
                        MARKER_LARGE_STRING => input_bytes.get_u32() as usize,
                        _ => {
                            return Err(DeserializationError::InvalidMarkerByte(marker).into());
                        }
                    };
                    let mut string_bytes = vec![0; size];
                    // We resize here so that the length of string_bytes is nonzero, which allows
                    // us to use copy_to_slice
                    input_bytes.copy_to_slice(&mut string_bytes);
                    Ok(Value::String(
                        std::string::String::from_utf8(string_bytes.to_vec())
                            .map_err(DeserializationError::InvalidUTF8)?,
                    ))
                }
                // Tiny structure
                marker if (MARKER_TINY_STRUCT..=(MARKER_TINY_STRUCT | 0x0F)).contains(&marker) => {
                    deserialize_structure(input_arc)
                }
                MARKER_SMALL_STRUCT | MARKER_MEDIUM_STRUCT => deserialize_structure(input_arc),
                _ => Err(DeserializationError::InvalidMarkerByte(marker).into()),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

fn deserialize_structure(input_arc: Arc<Mutex<Bytes>>) -> Result<Value> {
    let (_marker, signature) = get_info_from_bytes(input_arc.lock().unwrap().deref_mut())?;
    match signature {
        node::SIGNATURE => Ok(Value::Node(Node::try_from(input_arc)?)),
        relationship::SIGNATURE => Ok(Value::Relationship(Relationship::try_from(input_arc)?)),
        path::SIGNATURE => Ok(Value::Path(Path::try_from(input_arc)?)),
        unbound_relationship::SIGNATURE => Ok(Value::UnboundRelationship(
            UnboundRelationship::try_from(input_arc)?,
        )),
        SIGNATURE_DATE => {
            let days_since_epoch: i64 = Value::try_from(input_arc)?.try_into()?;
            Ok(Value::Date(
                NaiveDate::from_ymd(1970, 1, 1) + chrono::Duration::days(days_since_epoch),
            ))
        }
        SIGNATURE_TIME => {
            let nanos_since_midnight: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let zone_offset: i32 = Value::try_from(input_arc)?.try_into()?;
            Ok(Value::Time(
                NaiveTime::from_num_seconds_from_midnight(
                    (nanos_since_midnight / 1_000_000_000) as u32,
                    (nanos_since_midnight % 1_000_000_000) as u32,
                ),
                FixedOffset::east(zone_offset),
            ))
        }
        SIGNATURE_DATE_TIME_OFFSET => {
            let epoch_seconds: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let nanos: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let offset_seconds: i32 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            Ok(Value::DateTimeOffset(DateTime::from_utc(
                NaiveDateTime::from_timestamp(epoch_seconds, nanos as u32),
                FixedOffset::east(offset_seconds),
            )))
        }
        SIGNATURE_DATE_TIME_ZONED => {
            let epoch_seconds: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let nanos: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let timezone_id: String = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let timezone: Tz = timezone_id.parse().unwrap();
            Ok(Value::DateTimeZoned(
                timezone.timestamp(epoch_seconds, nanos as u32),
            ))
        }
        SIGNATURE_LOCAL_TIME => {
            let nanos_since_midnight: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            Ok(Value::LocalTime(NaiveTime::from_num_seconds_from_midnight(
                (nanos_since_midnight / 1_000_000_000) as u32,
                (nanos_since_midnight % 1_000_000_000) as u32,
            )))
        }
        SIGNATURE_LOCAL_DATE_TIME => {
            let epoch_seconds: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            let nanos: i64 = Value::try_from(Arc::clone(&input_arc))?.try_into()?;
            Ok(Value::LocalDateTime(NaiveDateTime::from_timestamp(
                epoch_seconds,
                nanos as u32,
            )))
        }
        duration::SIGNATURE => Ok(Value::Duration(Duration::try_from(input_arc)?)),
        point_2d::SIGNATURE => Ok(Value::Point2D(Point2D::try_from(input_arc)?)),
        point_3d::SIGNATURE => Ok(Value::Point3D(Point3D::try_from(input_arc)?)),
        _ => Err(DeserializationError::InvalidSignatureByte(signature).into()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::iter::{self, FromIterator};

    use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};

    use super::*;

    #[test]
    fn null_from_bytes() {
        let null_bytes = Bytes::from_static(&[MARKER_NULL]);
        assert_eq!(Value::Null.serialize().unwrap(), null_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(null_bytes))).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn boolean_from_bytes() {
        let true_bytes = Bytes::from_static(&[MARKER_TRUE]);
        let false_bytes = Bytes::from_static(&[MARKER_FALSE]);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(true_bytes))).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(false_bytes))).unwrap(),
            Value::Boolean(false)
        );
    }

    #[test]
    fn tiny_integer_from_bytes() {
        let tiny = Value::Integer(110);
        let tiny_bytes = Bytes::from_static(&[110]);
        assert_eq!(&tiny.clone().serialize().unwrap(), &tiny_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_bytes))).unwrap(),
            tiny
        );
    }

    #[test]
    fn small_integer_from_bytes() {
        let small = Value::Integer(-127);
        let small_bytes = Bytes::from_static(&[0xC8, 0x81]);
        assert_eq!(&small.clone().serialize().unwrap(), &small_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_bytes))).unwrap(),
            small
        );
    }

    #[test]
    fn medium_integer_from_bytes() {
        let medium = Value::Integer(8000);
        let medium_bytes = Bytes::from_static(&[0xC9, 0x1F, 0x40]);
        assert_eq!(&medium.clone().serialize().unwrap(), &medium_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_bytes))).unwrap(),
            medium
        );
    }

    #[test]
    fn medium_negative_integer_from_bytes() {
        let medium_negative = Value::Integer(-18621);
        let medium_negative_bytes = Bytes::from_static(&[0xC9, 0xB7, 0x43]);
        assert_eq!(
            &medium_negative.clone().serialize().unwrap(),
            &medium_negative_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_negative_bytes))).unwrap(),
            medium_negative
        );
    }

    #[test]
    fn large_integer_from_bytes() {
        let large = Value::Integer(-1_000_000_000);
        let large_bytes = Bytes::from_static(&[0xCA, 0xC4, 0x65, 0x36, 0x00]);
        assert_eq!(&large.clone().serialize().unwrap(), &large_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_bytes))).unwrap(),
            large
        );
    }

    #[test]
    fn very_large_integer_from_bytes() {
        let very_large = Value::Integer(9_000_000_000_000_000_000);
        let very_large_bytes =
            Bytes::from_static(&[0xCB, 0x7C, 0xE6, 0x6C, 0x50, 0xE2, 0x84, 0x00, 0x00]);
        assert_eq!(&very_large.clone().serialize().unwrap(), &very_large_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(very_large_bytes))).unwrap(),
            very_large
        );
    }

    #[test]
    fn float_from_bytes() {
        let min = Value::Float(std::f64::MIN_POSITIVE);
        let min_bytes = min.clone().serialize().unwrap();
        let max = Value::Float(std::f64::MAX);
        let max_bytes = max.clone().serialize().unwrap();
        let e = Value::Float(std::f64::consts::E);
        let e_bytes = e.clone().serialize().unwrap();
        let pi = Value::Float(std::f64::consts::PI);
        let pi_bytes = pi.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(min_bytes))).unwrap(),
            min
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(max_bytes))).unwrap(),
            max
        );
        assert_eq!(Value::try_from(Arc::new(Mutex::new(e_bytes))).unwrap(), e);
        assert_eq!(Value::try_from(Arc::new(Mutex::new(pi_bytes))).unwrap(), pi);
    }

    #[test]
    fn byte_array_from_bytes() {
        let empty_arr = Value::Bytes(vec![]);
        let empty_arr_bytes = empty_arr.clone().serialize().unwrap();
        let small_arr = Value::Bytes(vec![1_u8; 100]);
        let small_arr_bytes = small_arr.clone().serialize().unwrap();
        let medium_arr = Value::Bytes(vec![99_u8; 1000]);
        let medium_arr_bytes = medium_arr.clone().serialize().unwrap();
        let large_arr = Value::Bytes(vec![1_u8; 100_000]);
        let large_arr_bytes = large_arr.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(empty_arr_bytes))).unwrap(),
            empty_arr
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_arr_bytes))).unwrap(),
            small_arr
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_arr_bytes))).unwrap(),
            medium_arr
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_arr_bytes))).unwrap(),
            large_arr
        );
    }

    #[test]
    fn empty_list_from_bytes() {
        let empty_list = Value::List(vec![]);
        let empty_list_bytes = Bytes::from_static(&[MARKER_TINY_LIST | 0]);
        assert_eq!(&empty_list.clone().serialize().unwrap(), &empty_list_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(empty_list_bytes))).unwrap(),
            empty_list
        );
    }

    #[test]
    fn tiny_list_from_bytes() {
        let tiny_list = Value::from(vec![100_000; 3]);
        let tiny_list_bytes = Bytes::from_static(&[
            MARKER_TINY_LIST | 3,
            MARKER_INT_32,
            0x00,
            0x01,
            0x86,
            0xA0,
            MARKER_INT_32,
            0x00,
            0x01,
            0x86,
            0xA0,
            MARKER_INT_32,
            0x00,
            0x01,
            0x86,
            0xA0,
        ]);
        assert_eq!(&tiny_list.clone().serialize().unwrap(), &tiny_list_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_list_bytes))).unwrap(),
            tiny_list
        );
    }

    #[test]
    fn small_list_from_bytes() {
        let small_list = Value::from(vec!["item"; 100]);
        let small_list_bytes = Bytes::from_iter(
            vec![MARKER_SMALL_LIST, 100].into_iter().chain(
                iter::repeat(&[MARKER_TINY_STRING | 4, 0x69, 0x74, 0x65, 0x6D])
                    .take(100)
                    .flatten()
                    .copied(),
            ),
        );
        assert_eq!(&small_list.clone().serialize().unwrap(), &small_list_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_list_bytes))).unwrap(),
            small_list
        );
    }

    #[test]
    fn medium_list_from_bytes() {
        let medium_list = Value::from(vec![false; 1000]);
        let medium_list_bytes = Bytes::from_iter(
            vec![MARKER_MEDIUM_LIST, 0x03, 0xE8]
                .into_iter()
                .chain(vec![MARKER_FALSE; 1000]),
        );
        assert_eq!(
            &medium_list.clone().serialize().unwrap(),
            &medium_list_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_list_bytes))).unwrap(),
            medium_list
        );
    }

    #[test]
    #[ignore]
    fn large_list_from_bytes() {
        let large_list = Value::from(vec![1_i8; 70_000]);
        let large_list_bytes = Bytes::from_iter(
            vec![MARKER_LARGE_LIST, 0x00, 0x01, 0x11, 0x70]
                .into_iter()
                .chain(vec![1; 70_000]),
        );
        assert_eq!(&large_list.clone().serialize().unwrap(), &large_list_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_list_bytes))).unwrap(),
            large_list
        );
    }

    #[test]
    fn tiny_string_from_bytes() {
        let tiny = Value::from("string");
        let tiny_bytes =
            Bytes::from_static(&[MARKER_TINY_STRING | 6, b's', b't', b'r', b'i', b'n', b'g']);
        assert_eq!(&tiny.clone().serialize().unwrap(), &tiny_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_bytes))).unwrap(),
            tiny
        );
    }

    #[test]
    fn small_string_from_bytes() {
        let small = Value::from("string".repeat(10));
        let small_bytes = Bytes::from_iter(
            vec![MARKER_SMALL_STRING, 60].into_iter().chain(
                iter::repeat(&[b's', b't', b'r', b'i', b'n', b'g'])
                    .take(10)
                    .flatten()
                    .copied(),
            ),
        );
        assert_eq!(small.clone().serialize().unwrap(), &small_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_bytes))).unwrap(),
            small
        );
    }

    #[test]
    fn medium_string_from_bytes() {
        let medium = Value::from("string".repeat(1000));
        let medium_bytes = Bytes::from_iter(
            vec![MARKER_MEDIUM_STRING, 0x17, 0x70].into_iter().chain(
                iter::repeat(&[b's', b't', b'r', b'i', b'n', b'g'])
                    .take(1000)
                    .flatten()
                    .copied(),
            ),
        );
        assert_eq!(medium.clone().serialize().unwrap(), &medium_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(medium_bytes))).unwrap(),
            medium
        );
    }

    #[test]
    fn large_string_from_bytes() {
        let large = Value::from("string".repeat(100_000));
        let large_bytes = Bytes::from_iter(
            vec![MARKER_LARGE_STRING, 0x00, 0x09, 0x27, 0xC0]
                .into_iter()
                .chain(
                    iter::repeat(&[b's', b't', b'r', b'i', b'n', b'g'])
                        .take(100_000)
                        .flatten()
                        .copied(),
                ),
        );
        assert_eq!(large.clone().serialize().unwrap(), &large_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(large_bytes))).unwrap(),
            large
        );
    }

    #[test]
    fn special_string_from_bytes() {
        let special = Value::from("En å flöt över ängen");
        let special_bytes = Bytes::from_static(&[
            MARKER_SMALL_STRING,
            24,
            0x45,
            0x6e,
            0x20,
            0xc3,
            0xa5,
            0x20,
            0x66,
            0x6c,
            0xc3,
            0xb6,
            0x74,
            0x20,
            0xc3,
            0xb6,
            0x76,
            0x65,
            0x72,
            0x20,
            0xc3,
            0xa4,
            0x6e,
            0x67,
            0x65,
            0x6e,
        ]);
        assert_eq!(special.clone().serialize().unwrap(), &special_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(special_bytes))).unwrap(),
            special
        );
    }

    #[test]
    fn empty_map_from_bytes() {
        let empty_map = Value::from(HashMap::<&str, i8>::new());
        let empty_map_bytes = empty_map.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(empty_map_bytes))).unwrap(),
            empty_map
        );
    }

    #[test]
    fn tiny_map_from_bytes() {
        let tiny_map = Value::from(HashMap::<&str, i8>::from_iter(vec![("a", 1_i8)]));
        let tiny_map_bytes = tiny_map.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(tiny_map_bytes))).unwrap(),
            tiny_map
        );
    }

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
        let small_map_bytes = small_map.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(small_map_bytes))).unwrap(),
            small_map
        );
    }

    fn get_node() -> Node {
        Node::new(
            24_i64,
            vec!["TestNode".to_string()],
            HashMap::from_iter(vec![
                ("key1".to_string(), -1_i8),
                ("key2".to_string(), 1_i8),
            ]),
        )
    }

    fn get_rel() -> Relationship {
        Relationship::new(
            24_i64,
            32_i64,
            128_i64,
            "TestRel".to_string(),
            HashMap::from_iter(vec![
                ("key1".to_string(), -2_i8),
                ("key2".to_string(), 2_i8),
            ]),
        )
    }

    fn get_unbound_rel() -> UnboundRelationship {
        UnboundRelationship::new(
            128_i64,
            "TestRel".to_string(),
            HashMap::from_iter(vec![
                ("key1".to_string(), -2_i8),
                ("key2".to_string(), 2_i8),
            ]),
        )
    }

    #[test]
    fn node_from_bytes() {
        let node_bytes: Bytes = get_node().serialize().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(node_bytes))).unwrap(),
            Value::Node(get_node())
        );
    }

    #[test]
    fn relationship_from_bytes() {
        let rel_bytes: Bytes = get_rel().serialize().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(rel_bytes))).unwrap(),
            Value::Relationship(get_rel())
        );
    }

    #[test]
    fn path_from_bytes() {
        let path = Path::new(vec![get_node()], vec![get_unbound_rel()], vec![100, 101]);
        let path_bytes: Bytes = path.clone().serialize().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(path_bytes))).unwrap(),
            Value::Path(path)
        );
    }

    #[test]
    fn unbound_relationship_from_bytes() {
        let unbound_rel_bytes: Bytes = get_unbound_rel().serialize().unwrap();

        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(unbound_rel_bytes))).unwrap(),
            Value::UnboundRelationship(get_unbound_rel())
        );
    }

    #[test]
    fn date_from_bytes() {
        let christmas = Value::Date(NaiveDate::from_ymd(2020, 12, 25));
        let christmas_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 1,
            SIGNATURE_DATE,
            MARKER_INT_16,
            0x48,
            0xBD,
        ]);
        assert_eq!(&christmas.clone().serialize().unwrap(), &christmas_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(christmas_bytes))).unwrap(),
            christmas
        );
    }

    #[test]
    fn past_date_from_bytes() {
        let past_date = Value::Date(NaiveDate::from_ymd(1901, 12, 31));
        let past_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 1,
            SIGNATURE_DATE,
            MARKER_INT_16,
            0x9E,
            0xFA,
        ]);
        assert_eq!(&past_date.clone().serialize().unwrap(), &past_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(past_bytes))).unwrap(),
            past_date
        );
    }

    #[test]
    fn future_date_from_bytes() {
        let future_date = Value::Date(NaiveDate::from_ymd(3000, 5, 23));
        let future_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 1,
            SIGNATURE_DATE,
            MARKER_INT_32,
            0x00,
            0x05,
            0xBE,
            0x16,
        ]);
        assert_eq!(&future_date.clone().serialize().unwrap(), &future_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(future_bytes))).unwrap(),
            future_date
        );
    }

    #[test]
    fn time_from_bytes() {
        let midnight_utc = Value::Time(NaiveTime::from_hms_nano(0, 0, 0, 0), Utc.fix());
        let midnight_utc_bytes =
            Bytes::from_static(&[MARKER_TINY_STRUCT | 2, SIGNATURE_TIME, 0, 0]);
        assert_eq!(
            &midnight_utc.clone().serialize().unwrap(),
            &midnight_utc_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(midnight_utc_bytes))).unwrap(),
            midnight_utc
        );

        let about_four_pm_pacific = Value::Time(
            NaiveTime::from_hms_nano(16, 4, 35, 235),
            FixedOffset::east(-8 * 3600),
        );
        let about_four_pm_pacific_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 2,
            SIGNATURE_TIME,
            MARKER_INT_64,
            0x00,
            0x00,
            0x34,
            0xA3,
            0x12,
            0xD0,
            0xFE,
            0xEB,
            MARKER_INT_16,
            0x8F,
            0x80,
        ]);
        assert_eq!(
            &about_four_pm_pacific.clone().serialize().unwrap(),
            &about_four_pm_pacific_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(about_four_pm_pacific_bytes))).unwrap(),
            about_four_pm_pacific
        );
    }

    #[test]
    fn date_time_offset_from_bytes() {
        let date_time = Value::DateTimeOffset(
            FixedOffset::east(-5 * 3600)
                .from_utc_datetime(&NaiveDate::from_ymd(2050, 12, 31).and_hms_nano(23, 59, 59, 10)),
        );
        let date_time_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 3,
            SIGNATURE_DATE_TIME_OFFSET,
            MARKER_INT_64,
            0x00,
            0x00,
            0x00,
            0x00,
            0x98,
            0x5B,
            0xA9,
            0x7F,
            10,
            MARKER_INT_16,
            0xB9,
            0xB0,
        ]);
        assert_eq!(&date_time.clone().serialize().unwrap(), &date_time_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(date_time_bytes))).unwrap(),
            date_time
        );
    }

    #[test]
    fn date_time_zoned_from_bytes() {
        let date_time = Value::DateTimeZoned(
            chrono_tz::Asia::Ulaanbaatar
                .ymd(2030, 8, 3)
                .and_hms_milli(14, 30, 1, 2),
        );
        let date_time_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 3,
            SIGNATURE_DATE_TIME_ZONED,
            MARKER_INT_32,
            0x71,
            0xF6,
            0x54,
            0xE9,
            MARKER_INT_32,
            0x00,
            0x1E,
            0x84,
            0x80,
            MARKER_SMALL_STRING,
            16,
            b'A',
            b's',
            b'i',
            b'a',
            b'/',
            b'U',
            b'l',
            b'a',
            b'a',
            b'n',
            b'b',
            b'a',
            b'a',
            b't',
            b'a',
            b'r',
        ]);
        assert_eq!(&date_time.clone().serialize().unwrap(), &date_time_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(date_time_bytes))).unwrap(),
            date_time
        );
    }

    #[test]
    fn local_time_from_bytes() {
        let local_time = Value::LocalTime(NaiveTime::from_hms_nano(23, 59, 59, 999));
        let local_time_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 1,
            SIGNATURE_LOCAL_TIME,
            MARKER_INT_64,
            0x00,
            0x00,
            0x4E,
            0x94,
            0x55,
            0xB4,
            0x39,
            0xE7,
        ]);
        assert_eq!(&local_time.clone().serialize().unwrap(), &local_time_bytes);
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(local_time_bytes))).unwrap(),
            local_time
        );
    }

    #[test]
    fn local_date_time_from_bytes() {
        let local_date_time =
            Value::LocalDateTime(NaiveDate::from_ymd(1999, 2, 27).and_hms_nano(1, 0, 0, 9999));
        let local_date_time_bytes = Bytes::from_static(&[
            MARKER_TINY_STRUCT | 2,
            SIGNATURE_LOCAL_DATE_TIME,
            MARKER_INT_32,
            0x36,
            0xD7,
            0x43,
            0x90,
            MARKER_INT_16,
            0x27,
            0x0F,
        ]);
        assert_eq!(
            &local_date_time.clone().serialize().unwrap(),
            &local_date_time_bytes
        );
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(local_date_time_bytes))).unwrap(),
            local_date_time
        );
    }

    #[test]
    fn duration_from_bytes() {
        let duration = Duration::new(9876, 12345, 65332, 23435);
        let duration_bytes = duration.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(duration_bytes))).unwrap(),
            Value::Duration(duration)
        );
    }

    #[test]
    fn point_from_bytes() {
        let point2d = Point2D::new(9876, 12.312_345, 134_564.123_567_543);
        let point2d_bytes = point2d.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(point2d_bytes))).unwrap(),
            Value::Point2D(point2d)
        );

        let point3d = Point3D::new(249, 543.598_387, 2_945_732_849.293_85, 45_438.874_385);
        let point3d_bytes = point3d.clone().serialize().unwrap();
        assert_eq!(
            Value::try_from(Arc::new(Mutex::new(point3d_bytes))).unwrap(),
            Value::Point3D(point3d)
        );
    }

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
