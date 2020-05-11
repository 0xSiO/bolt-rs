use std::convert::{TryFrom, TryInto};
use std::mem;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::io::BufStream;
use tokio::prelude::*;

// TODO: Do these need to be pub used?
pub use ack_failure::AckFailure;
pub use begin::Begin;
pub(crate) use chunk::Chunk;
pub use commit::Commit;
pub use discard_all::DiscardAll;
pub use failure::Failure;
pub use goodbye::Goodbye;
pub use hello::Hello;
pub use ignored::Ignored;
pub use init::Init;
pub(crate) use message_bytes::MessageBytes;
pub use pull::Pull;
pub use pull_all::PullAll;
pub use record::Record;
pub use reset::Reset;
pub use rollback::Rollback;
pub use run::Run;
pub use run_with_metadata::RunWithMetadata;
pub use success::Success;

use crate::error::*;
use crate::serialization::*;

pub(crate) mod ack_failure;
pub(crate) mod begin;
pub(crate) mod commit;
pub(crate) mod discard_all;
pub(crate) mod failure;
pub(crate) mod goodbye;
pub(crate) mod hello;
pub(crate) mod ignored;
pub(crate) mod init;
pub(crate) mod pull;
pub(crate) mod pull_all;
pub(crate) mod record;
pub(crate) mod reset;
pub(crate) mod rollback;
pub(crate) mod run;
pub(crate) mod run_with_metadata;
pub(crate) mod success;

mod chunk;
mod message_bytes;

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
    // Discard,
    Pull(Pull),
}

impl Message {
    pub async fn from_stream<T: Unpin + AsyncRead + AsyncWrite>(
        buf_stream: &mut BufStream<T>,
    ) -> Result<Message> {
        Message::try_from(MessageBytes::from_stream(buf_stream).await?)
    }
}

impl Marker for Message {
    fn get_marker(&self) -> Result<u8> {
        match self {
            Message::Init(init) => init.get_marker(),
            Message::Run(run) => run.get_marker(),
            Message::DiscardAll => DiscardAll.get_marker(),
            Message::PullAll => PullAll.get_marker(),
            Message::AckFailure => AckFailure.get_marker(),
            Message::Reset => Reset.get_marker(),
            Message::Record(record) => record.get_marker(),
            Message::Success(success) => success.get_marker(),
            Message::Failure(failure) => failure.get_marker(),
            Message::Ignored => Ignored.get_marker(),
            Message::Hello(hello) => hello.get_marker(),
            Message::Goodbye => Goodbye.get_marker(),
            Message::RunWithMetadata(run_with_metadata) => run_with_metadata.get_marker(),
            Message::Begin(begin) => begin.get_marker(),
            Message::Commit => Commit.get_marker(),
            Message::Rollback => Rollback.get_marker(),
            Message::Pull(pull) => pull.get_marker(),
        }
    }
}

impl Signature for Message {
    fn get_signature(&self) -> u8 {
        match self {
            Message::Init(init) => init.get_signature(),
            Message::Run(run) => run.get_signature(),
            Message::DiscardAll => DiscardAll.get_signature(),
            Message::PullAll => PullAll.get_signature(),
            Message::AckFailure => AckFailure.get_signature(),
            Message::Reset => Reset.get_signature(),
            Message::Record(record) => record.get_signature(),
            Message::Success(success) => success.get_signature(),
            Message::Failure(failure) => failure.get_signature(),
            Message::Ignored => Ignored.get_signature(),
            Message::Hello(hello) => hello.get_signature(),
            Message::Goodbye => Goodbye.get_signature(),
            Message::RunWithMetadata(run_with_metadata) => run_with_metadata.get_signature(),
            Message::Begin(begin) => begin.get_signature(),
            Message::Commit => Commit.get_signature(),
            Message::Rollback => Rollback.get_signature(),
            Message::Pull(pull) => pull.get_signature(),
        }
    }
}

impl Serialize for Message {}

impl TryInto<Bytes> for Message {
    type Error = Error;

    fn try_into(self) -> Result<Bytes> {
        match self {
            Message::Init(init) => init.try_into(),
            Message::Run(run) => run.try_into(),
            Message::DiscardAll => DiscardAll.try_into(),
            Message::PullAll => PullAll.try_into(),
            Message::AckFailure => AckFailure.try_into(),
            Message::Reset => Reset.try_into(),
            Message::Record(record) => record.try_into(),
            Message::Success(success) => success.try_into(),
            Message::Failure(failure) => failure.try_into(),
            Message::Ignored => Ignored.try_into(),
            Message::Hello(hello) => hello.try_into(),
            Message::Goodbye => Goodbye.try_into(),
            Message::RunWithMetadata(run_with_metadata) => run_with_metadata.try_into(),
            Message::Begin(begin) => begin.try_into(),
            Message::Commit => Commit.try_into(),
            Message::Rollback => Rollback.try_into(),
            Message::Pull(pull) => pull.try_into(),
        }
    }
}

impl Deserialize for Message {}

impl TryFrom<Arc<Mutex<Bytes>>> for Message {
    type Error = Error;

    fn try_from(value: Arc<Mutex<Bytes>>) -> Result<Self> {
        let message_bytes = MessageBytes::try_from(value)?;
        Message::try_from(message_bytes)
    }
}

impl TryFrom<MessageBytes> for Message {
    type Error = Error;

    fn try_from(mut message_bytes: MessageBytes) -> Result<Self> {
        catch_unwind(move || {
            let (marker, signature) = get_info_from_bytes(&mut message_bytes)?;
            let remaining_bytes_arc =
                Arc::new(Mutex::new(message_bytes.split_to(message_bytes.len())));

            match signature {
                init::SIGNATURE => {
                    // Equal to hello::SIGNATURE, so we have to check for metadata.
                    // INIT has 2 fields, while HELLO has 1.
                    if marker == init::MARKER {
                        Ok(Message::Init(Init::try_from(remaining_bytes_arc)?))
                    } else {
                        Ok(Message::Hello(Hello::try_from(remaining_bytes_arc)?))
                    }
                }
                run::SIGNATURE => {
                    // Equal to run_with_metadata::SIGNATURE, so we have to check for metadata.
                    // RUN has 2 fields, while RUN_WITH_METADATA has 3.
                    if marker == run::MARKER {
                        Ok(Message::Run(Run::try_from(remaining_bytes_arc)?))
                    } else {
                        Ok(Message::RunWithMetadata(RunWithMetadata::try_from(
                            remaining_bytes_arc,
                        )?))
                    }
                }
                discard_all::SIGNATURE => Ok(Message::DiscardAll),
                pull_all::SIGNATURE => {
                    // Equal to pull::SIGNATURE, so we have to check for metadata.
                    // PULL_ALL has 0 fields, while PULL has 1.
                    if marker == pull_all::MARKER {
                        Ok(Message::PullAll)
                    } else {
                        Ok(Message::Pull(Pull::try_from(remaining_bytes_arc)?))
                    }
                }
                ack_failure::SIGNATURE => Ok(Message::AckFailure),
                reset::SIGNATURE => Ok(Message::Reset),
                record::SIGNATURE => Ok(Message::Record(Record::try_from(remaining_bytes_arc)?)),
                success::SIGNATURE => Ok(Message::Success(Success::try_from(remaining_bytes_arc)?)),
                failure::SIGNATURE => Ok(Message::Failure(Failure::try_from(remaining_bytes_arc)?)),
                ignored::SIGNATURE => Ok(Message::Ignored),
                goodbye::SIGNATURE => Ok(Message::Goodbye),
                begin::SIGNATURE => Ok(Message::Begin(Begin::try_from(remaining_bytes_arc)?)),
                commit::SIGNATURE => Ok(Message::Commit),
                rollback::SIGNATURE => Ok(Message::Rollback),
                _ => Err(DeserializationError::InvalidSignatureByte(signature).into()),
            }
        })
        .map_err(|_| DeserializationError::Panicked)?
    }
}

impl TryInto<Vec<Bytes>> for Message {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Bytes>> {
        let bytes: Bytes = self.try_into_bytes()?;

        // Big enough to hold all the chunks, plus a partial chunk, plus the message footer
        let mut result: Vec<Bytes> = Vec::with_capacity(bytes.len() / CHUNK_SIZE + 2);
        for slice in bytes.chunks(CHUNK_SIZE) {
            let chunk_bytes: Bytes = Chunk::try_from(Bytes::copy_from_slice(slice))?.into();
            result.push(chunk_bytes);
        }
        // End message
        result.push(Bytes::from_static(&[0, 0]));

        Ok(result)
    }
}
