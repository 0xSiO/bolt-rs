use crate::serialize::Serialize;
use crate::value::Marker;

pub trait Structure: Marker + Serialize {
    fn get_signature(&self) -> u8;
}

// TODO: Create Structure enum to hold protocol Message types
