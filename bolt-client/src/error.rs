use failure::{Fail, Fallible};

pub(crate) type Result<T> = Fallible<T>;

#[derive(Debug, Fail)]
pub(crate) enum ClientError {
    #[fail(display = "Connection to server failed")]
    ConnectionFailed,
    #[fail(display = "Unsupported operation for Bolt v{}", _0)]
    UnsupportedOperation(u32),
}
