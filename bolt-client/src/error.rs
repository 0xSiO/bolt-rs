use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid DNS name: {0}")]
    InvalidDNSName(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Handshake with server failed for versions {0:?}")]
    HandshakeFailed([u32; 4]),
    #[error("Unsupported operation for client with version = {0:?}")]
    UnsupportedOperation(Option<u32>),
    #[error(transparent)]
    ProtocolError(#[from] bolt_proto::error::Error),
}
