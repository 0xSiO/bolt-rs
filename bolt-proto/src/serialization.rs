use std::panic::UnwindSafe;

use bytes::{Buf, Bytes};

use crate::{
    error::*,
    value::{MARKER_MEDIUM_STRUCT, MARKER_SMALL_STRUCT, MARKER_TINY_STRUCT},
};

pub(crate) trait BoltValue: Sized {
    fn marker(&self) -> MarkerResult<u8>;

    fn serialize(self) -> SerializeResult<Bytes>;

    fn deserialize<B: Buf + UnwindSafe>(bytes: B) -> DeserializeResult<(Self, B)>;
}

pub(crate) trait BoltStructure: BoltValue {
    fn signature(&self) -> u8;
}

/// Returns marker, size, and signature. Might panic - use this inside a catch_unwind block
pub(crate) fn get_structure_info(bytes: &mut impl Buf) -> DeserializeResult<(u8, usize, u8)> {
    let marker = bytes.get_u8();
    let size = match marker {
        marker if (MARKER_TINY_STRUCT..=(MARKER_TINY_STRUCT | 0x0F)).contains(&marker) => {
            0x0F & marker as usize
        }
        MARKER_SMALL_STRUCT => bytes.get_u8() as usize,
        MARKER_MEDIUM_STRUCT => bytes.get_u16() as usize,
        _ => {
            return Err(DeserializationError::InvalidMarkerByte(marker));
        }
    };
    let signature = bytes.get_u8();
    Ok((marker, size, signature))
}
