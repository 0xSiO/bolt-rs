use std::{
    mem,
    panic::{catch_unwind, UnwindSafe},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_util::io::{AsyncRead, AsyncReadExt};

pub use begin::Begin;
pub use discard::Discard;
pub use failure::Failure;
pub use hello::Hello;
pub use init::Init;
pub use pull::Pull;
pub use record::Record;
pub use run::Run;
pub use run_with_metadata::RunWithMetadata;
pub use success::Success;

use crate::{error::*, serialization::*, value::MARKER_TINY_STRUCT};

pub(crate) mod begin;
pub(crate) mod discard;
pub(crate) mod failure;
pub(crate) mod hello;
pub(crate) mod init;
pub(crate) mod pull;
pub(crate) mod record;
pub(crate) mod run;
pub(crate) mod run_with_metadata;
pub(crate) mod success;

pub(crate) const SIGNATURE_INIT: u8 = 0x01;
pub(crate) const SIGNATURE_RUN: u8 = 0x10;
pub(crate) const SIGNATURE_DISCARD_ALL: u8 = 0x2F;
pub(crate) const SIGNATURE_PULL_ALL: u8 = 0x3F;
pub(crate) const SIGNATURE_ACK_FAILURE: u8 = 0x0E;
pub(crate) const SIGNATURE_RESET: u8 = 0x0F;
pub(crate) const SIGNATURE_RECORD: u8 = 0x71;
pub(crate) const SIGNATURE_SUCCESS: u8 = 0x70;
pub(crate) const SIGNATURE_FAILURE: u8 = 0x7F;
pub(crate) const SIGNATURE_IGNORED: u8 = 0x7E;
pub(crate) const SIGNATURE_HELLO: u8 = 0x01;
pub(crate) const SIGNATURE_GOODBYE: u8 = 0x02;
pub(crate) const SIGNATURE_RUN_WITH_METADATA: u8 = 0x10;
pub(crate) const SIGNATURE_BEGIN: u8 = 0x11;
pub(crate) const SIGNATURE_COMMIT: u8 = 0x12;
pub(crate) const SIGNATURE_ROLLBACK: u8 = 0x13;
pub(crate) const SIGNATURE_DISCARD: u8 = 0x2F;
pub(crate) const SIGNATURE_PULL: u8 = 0x3F;

// This is the default maximum chunk size in the official driver, minus header length
const CHUNK_SIZE: usize = 16383 - mem::size_of::<u16>();

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    // V1-compatible message types
    Init(Init),
    Run(Run),
    DiscardAll,
    PullAll,
    AckFailure,
    Reset,
    Record(Record),
    Success(Success),
    Failure(Failure),
    Ignored,

    // V3+-compatible message types
    Hello(Hello),
    Goodbye,
    RunWithMetadata(RunWithMetadata),
    Begin(Begin),
    Commit,
    Rollback,

    // V4+-compatible message types
    Discard(Discard),
    Pull(Pull),
}

impl Message {
    pub async fn from_stream(mut stream: impl AsyncRead + Unpin) -> DeserializeResult<Message> {
        let mut bytes = BytesMut::new();
        let mut chunk_len = 0;
        // Ignore any no-op messages
        while chunk_len == 0 {
            let mut u16_bytes = [0, 0];
            stream.read_exact(&mut u16_bytes).await?;
            chunk_len = u16::from_be_bytes(u16_bytes);
        }
        // Messages end in a 0_u16
        while chunk_len > 0 {
            let mut buf = vec![0; chunk_len as usize];
            stream.read_exact(&mut buf).await?;
            bytes.put_slice(&buf);
            let mut u16_bytes = [0, 0];
            stream.read_exact(&mut u16_bytes).await?;
            chunk_len = u16::from_be_bytes(u16_bytes);
        }
        let (message, remaining) = Message::deserialize(bytes)?;
        debug_assert_eq!(remaining.len(), 0);

        Ok(message)
    }

    pub fn into_chunks(self) -> SerializeResult<Vec<Bytes>> {
        let bytes = self.serialize()?;

        // Big enough to hold all the chunks, plus a partial chunk, plus the message
        // footer
        let mut result: Vec<Bytes> = Vec::with_capacity(bytes.len() / CHUNK_SIZE + 2);
        for slice in bytes.chunks(CHUNK_SIZE) {
            // 16-bit size, then the chunk data
            let mut chunk = BytesMut::with_capacity(mem::size_of::<u16>() + slice.len());
            // Length of slice is at most CHUNK_SIZE, which can fit in a u16
            chunk.put_u16(slice.len() as u16);
            chunk.put(slice);
            result.push(chunk.freeze());
        }
        // End message
        result.push(Bytes::from_static(&[0, 0]));

        Ok(result)
    }
}

macro_rules! deserialize_struct {
    ($name:ident, $bytes:ident) => {{
        let (message, remaining) = $name::deserialize($bytes)?;
        $bytes = remaining;
        Ok((Message::$name(message), $bytes))
    }};
}

impl BoltValue for Message {
    fn marker(&self) -> SerializeResult<u8> {
        match self {
            Message::Init(init) => init.marker(),
            Message::Run(run) => run.marker(),
            Message::Record(record) => record.marker(),
            Message::Success(success) => success.marker(),
            Message::Failure(failure) => failure.marker(),
            Message::Hello(hello) => hello.marker(),
            Message::RunWithMetadata(run_with_metadata) => run_with_metadata.marker(),
            Message::Begin(begin) => begin.marker(),
            Message::Discard(discard) => discard.marker(),
            Message::Pull(pull) => pull.marker(),
            _ => Ok(MARKER_TINY_STRUCT),
        }
    }

    fn serialize(self) -> SerializeResult<Bytes> {
        match self {
            Message::Init(init) => init.serialize(),
            Message::Run(run) => run.serialize(),
            Message::Record(record) => record.serialize(),
            Message::Success(success) => success.serialize(),
            Message::Failure(failure) => failure.serialize(),
            Message::Hello(hello) => hello.serialize(),
            Message::RunWithMetadata(run_with_metadata) => run_with_metadata.serialize(),
            Message::Begin(begin) => begin.serialize(),
            Message::Discard(discard) => discard.serialize(),
            Message::Pull(pull) => pull.serialize(),
            other => Ok(Bytes::from(vec![other.marker()?, other.signature()])),
        }
    }

    fn deserialize<B: Buf + UnwindSafe>(mut bytes: B) -> DeserializeResult<(Self, B)> {
        catch_unwind(move || {
            let marker = bytes.get_u8();
            let (size, signature) = get_structure_info(marker, &mut bytes)?;

            match signature {
                SIGNATURE_INIT => {
                    // Conflicting signatures, so we have to check for metadata.
                    // HELLO has 1 field, while INIT has 2.
                    match size {
                        1 => deserialize_struct!(Hello, bytes),
                        2 => deserialize_struct!(Init, bytes),
                        _ => Err(DeserializationError::InvalidSize { size, signature }),
                    }
                }
                SIGNATURE_RUN => {
                    // Conflicting signatures, so we have to check for metadata.
                    // RUN has 2 fields, while RUN_WITH_METADATA has 3.
                    match size {
                        2 => deserialize_struct!(Run, bytes),
                        3 => deserialize_struct!(RunWithMetadata, bytes),
                        _ => Err(DeserializationError::InvalidSize { size, signature }),
                    }
                }
                SIGNATURE_DISCARD_ALL => {
                    // Conflicting signatures, so we have to check for metadata.
                    // DISCARD_ALL has 0 fields, while DISCARD has 1.
                    match size {
                        0 => Ok((Message::DiscardAll, bytes)),
                        1 => deserialize_struct!(Discard, bytes),
                        _ => Err(DeserializationError::InvalidSize { size, signature }),
                    }
                }
                SIGNATURE_PULL_ALL => {
                    // Conflicting signatures, so we have to check for metadata.
                    // PULL_ALL has 0 fields, while PULL has 1.
                    match size {
                        0 => Ok((Message::PullAll, bytes)),
                        1 => deserialize_struct!(Pull, bytes),
                        _ => Err(DeserializationError::InvalidSize { size, signature }),
                    }
                }
                SIGNATURE_ACK_FAILURE => Ok((Message::AckFailure, bytes)),
                SIGNATURE_RESET => Ok((Message::Reset, bytes)),
                SIGNATURE_RECORD => deserialize_struct!(Record, bytes),
                SIGNATURE_SUCCESS => deserialize_struct!(Success, bytes),
                SIGNATURE_FAILURE => deserialize_struct!(Failure, bytes),
                SIGNATURE_IGNORED => Ok((Message::Ignored, bytes)),
                SIGNATURE_GOODBYE => Ok((Message::Goodbye, bytes)),
                SIGNATURE_BEGIN => deserialize_struct!(Begin, bytes),
                SIGNATURE_COMMIT => Ok((Message::Commit, bytes)),
                SIGNATURE_ROLLBACK => Ok((Message::Rollback, bytes)),
                _ => Err(DeserializationError::InvalidSignatureByte(signature)),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

impl BoltStructure for Message {
    fn signature(&self) -> u8 {
        match self {
            Message::Init(_) => SIGNATURE_INIT,
            Message::Run(_) => SIGNATURE_RUN,
            Message::DiscardAll => SIGNATURE_DISCARD_ALL,
            Message::PullAll => SIGNATURE_PULL_ALL,
            Message::AckFailure => SIGNATURE_ACK_FAILURE,
            Message::Reset => SIGNATURE_RESET,
            Message::Record(_) => SIGNATURE_RECORD,
            Message::Success(_) => SIGNATURE_SUCCESS,
            Message::Failure(_) => SIGNATURE_FAILURE,
            Message::Ignored => SIGNATURE_IGNORED,
            Message::Hello(_) => SIGNATURE_HELLO,
            Message::Goodbye => SIGNATURE_GOODBYE,
            Message::RunWithMetadata(_) => SIGNATURE_RUN_WITH_METADATA,
            Message::Begin(_) => SIGNATURE_BEGIN,
            Message::Commit => SIGNATURE_COMMIT,
            Message::Rollback => SIGNATURE_ROLLBACK,
            Message::Discard(_) => SIGNATURE_DISCARD,
            Message::Pull(_) => SIGNATURE_PULL,
        }
    }
}
