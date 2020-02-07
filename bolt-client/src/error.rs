use failure::{Fail, Fallible};

pub(crate) type Result<T> = Fallible<T>;

#[derive(Debug, Fail)]
pub(crate) enum ClientError {
    #[fail(display = "Server does not support Bolt v{} clients", _0)]
    UnsupportedClientVersion(u8),
}
