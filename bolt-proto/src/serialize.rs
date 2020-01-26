use std::convert::{TryFrom, TryInto};
use std::sync::{Arc, Mutex};

use bytes::Bytes;

use crate::error::*;

pub trait Serialize: TryInto<Bytes, Error = Error> {
    fn try_into_bytes(self) -> Result<Bytes> {
        self.try_into()
    }
}

pub trait Deserialize: TryFrom<Arc<Mutex<Bytes>>, Error = Error> {}
