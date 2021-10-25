use bolt_proto::{error::Error as ProtocolError, Message, ServerState};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type ConnectionResult<T> = std::result::Result<T, ConnectionError>;
pub type CommunicationResult<T> = std::result::Result<T, CommunicationError>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ConnectionError(#[from] ConnectionError),
    #[error(transparent)]
    CommunicationError(Box<CommunicationError>),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
}

impl From<CommunicationError> for Error {
    fn from(error: CommunicationError) -> Self {
        Error::CommunicationError(Box::new(error))
    }
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("handshake with server failed for versions [{}]", format_versions(.0))]
    HandshakeFailed([u32; 4]),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error(
        "server gave unexpected response while in {state:?} state.
request: {request:?}
response: {response:?}"
    )]
    InvalidResponse {
        state: ServerState,
        request: Option<Message>,
        response: Message,
    },
    #[error("unsupported operation for server in {state:?} state: {message:?}")]
    InvalidState {
        state: ServerState,
        message: Message,
    },
    #[error("unsupported operation for client with version = {}", format_version(*.0))]
    UnsupportedOperation(u32),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

fn format_version(version: u32) -> String {
    let (major, minor, range) = (version & 0xff, version >> 8 & 0xff, version >> 16 & 0xff);
    if range > 0 {
        format!("{}.{}-{}", major, minor.saturating_sub(range), minor)
    } else {
        format!("{}.{}", major, minor)
    }
}

fn format_versions(versions: &[u32]) -> String {
    versions
        .iter()
        .map(|&v| format_version(v))
        .collect::<Vec<String>>()
        .join(", ")
}
