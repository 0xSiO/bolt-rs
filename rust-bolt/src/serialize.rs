use std::convert::TryInto;

use bytes::Bytes;
use failure::Error;

pub trait Serialize: TryInto<Bytes, Error = Error> {}
