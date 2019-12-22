use crate::bolt::value::Marker;
use crate::serialize::Serialize;

pub trait Structure: Marker + Serialize {
    fn get_signature(&self) -> u8;
}

// TODO: Create Structure enum to hold protocol Message types
