use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid DNS name: {0}")]
    InvalidDNSName(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Handshake with server failed")]
    HandshakeFailed,
    #[error("Unsupported operation for Bolt v{0}")]
    UnsupportedOperation(u32),
    #[error(transparent)]
    ProtocolError(#[from] bolt_proto::error::Error),
}
