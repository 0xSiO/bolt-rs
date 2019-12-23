use bytes::Buf;
use failure::Error;

use crate::bolt::value::Marker;
use crate::error::DeserializeError;

pub const MARKER_TINY: u8 = 0xB0;
pub const MARKER_SMALL: u8 = 0xDC;
pub const MARKER_MEDIUM: u8 = 0xDD;

pub trait Signature: Marker {
    fn get_signature(&self) -> u8;
}

// Might panic. Use this inside a catch_unwind block
pub fn get_signature_from_bytes(bytes: &mut dyn Buf) -> Result<u8, Error> {
    let marker = bytes.get_u8();
    let _size = match marker {
        marker if (MARKER_TINY..=(MARKER_TINY | 0x0F)).contains(&marker) => 0x0F & marker as usize,
        MARKER_SMALL => bytes.get_u8() as usize,
        MARKER_MEDIUM => bytes.get_u16() as usize,
        _ => {
            return Err(DeserializeError(format!("Invalid marker byte: {:x}", marker)).into());
        }
    };
    let signature = bytes.get_u8();
    Ok(signature)
}
