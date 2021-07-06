use std::{
    convert::{TryFrom, TryInto},
    panic::UnwindSafe,
    sync::{Arc, Mutex},
};

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

pub(crate) trait Serialize: TryInto<Bytes, Error = Error> {
    fn try_into_bytes(self) -> Result<Bytes> {
        self.try_into()
    }
}

pub(crate) trait Deserialize: TryFrom<Arc<Mutex<Bytes>>, Error = Error> {}

pub(crate) trait Marker {
    fn get_marker(&self) -> Result<u8>;
}

pub(crate) trait Signature {
    fn get_signature(&self) -> u8;
}

// Might panic. Use this inside a catch_unwind block
pub(crate) fn get_info_from_bytes(bytes: &mut impl Buf) -> Result<(u8, u8)> {
    let marker = bytes.get_u8();
    let _size = match marker {
        marker if (MARKER_TINY_STRUCT..=(MARKER_TINY_STRUCT | 0x0F)).contains(&marker) => {
            0x0F & marker as usize
        }
        MARKER_SMALL_STRUCT => bytes.get_u8() as usize,
        MARKER_MEDIUM_STRUCT => bytes.get_u16() as usize,
        _ => {
            return Err(DeserializationError::InvalidMarkerByte(marker).into());
        }
    };
    let signature = bytes.get_u8();
    Ok((marker, signature))
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
