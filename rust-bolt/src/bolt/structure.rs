use crate::bolt::value::Marker;

pub const MARKER_TINY: u8 = 0xB0;
pub const MARKER_SMALL: u8 = 0xDC;
pub const MARKER_MEDIUM: u8 = 0xDD;

pub trait Signature: Marker {
    fn get_signature(&self) -> u8;
}
