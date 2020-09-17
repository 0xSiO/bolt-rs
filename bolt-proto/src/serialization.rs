use std::convert::{TryFrom, TryInto};
use std::sync::{Arc, Mutex};

use bytes::{Buf, Bytes};

use crate::error::*;

pub trait Serialize: TryInto<Bytes, Error = Error> {
    fn try_into_bytes(self) -> Result<Bytes> {
        self.try_into()
    }
}

pub trait Deserialize: TryFrom<Arc<Mutex<Bytes>>, Error = Error> {}

pub trait Marker {
    fn get_marker(&self) -> Result<u8>;
}

pub trait Signature {
    fn get_signature(&self) -> u8;
}

pub(crate) const STRUCT_MARKER_TINY: u8 = 0xB0;
pub(crate) const STRUCT_MARKER_SMALL: u8 = 0xDC;
pub(crate) const STRUCT_MARKER_MEDIUM: u8 = 0xDD;

// Might panic. Use this inside a catch_unwind block
pub(crate) fn get_info_from_bytes(bytes: &mut impl Buf) -> Result<(u8, u8)> {
    let marker = bytes.get_u8();
    let _size = match marker {
        marker if (STRUCT_MARKER_TINY..=(STRUCT_MARKER_TINY | 0x0F)).contains(&marker) => {
            0x0F & marker as usize
        }
        STRUCT_MARKER_SMALL => bytes.get_u8() as usize,
        STRUCT_MARKER_MEDIUM => bytes.get_u16() as usize,
        _ => {
            return Err(DeserializationError::InvalidMarkerByte(marker).into());
        }
    };
    let signature = bytes.get_u8();
    Ok((marker, signature))
}
