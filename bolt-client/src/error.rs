use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Handshake with server failed")]
    HandshakeFailed,
    #[error("Unsupported operation for Bolt v{0}")]
    UnsupportedOperation(u32),
}
