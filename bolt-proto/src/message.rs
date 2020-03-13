use std::convert::{TryFrom, TryInto};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::io::BufStream;
use tokio::prelude::*;

pub use ack_failure::AckFailure;
pub(crate) use chunk::Chunk;
pub use discard_all::DiscardAll;
pub use failure::Failure;
pub use hello::Hello;
pub use ignored::Ignored;
pub use init::Init;
pub(crate) use message_bytes::MessageBytes;
pub use pull_all::PullAll;
pub use record::Record;
pub use reset::Reset;
pub use run::Run;
pub use success::Success;

use crate::error::*;
use crate::serialization::*;
use crate::value::*;

pub(crate) mod ack_failure;
pub(crate) mod discard_all;
pub(crate) mod failure;
pub(crate) mod hello;
pub(crate) mod ignored;
pub(crate) mod init;
pub(crate) mod pull_all;
pub(crate) mod record;
pub(crate) mod reset;
pub(crate) mod run;
pub(crate) mod success;

mod chunk;
mod message_bytes;

// This is what's used in the protocol spec, but it could technically be any size.
const CHUNK_SIZE: usize = 16; // TODO: Make this configurable

#[derive(Debug, Clone)]
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
            let signature = get_signature_from_bytes(&mut message_bytes)?;
            let remaining_bytes_arc =
                Arc::new(Mutex::new(message_bytes.split_to(message_bytes.len())));

            match signature {
                init::SIGNATURE => {
                    // Equal to hello::SIGNATURE, so we have to check for metadata. The way we'll do this is to check
                    // for the existence of a string (indicating it's an INIT), if not, then it's assumed to be a HELLO

                    // I confirmed that this peeking doesn't advance any buffer cursors
                    let next_byte = remaining_bytes_arc.lock().unwrap().as_ref()[0];

                    match next_byte {
                        // Tiny string
                        marker
                            if (string::MARKER_TINY..=(string::MARKER_TINY | 0x0F))
                                .contains(&marker) =>
                        {
                            Ok(Message::Init(Init::try_from(remaining_bytes_arc)?))
                        }
                        // Other string sizes
                        string::MARKER_SMALL | string::MARKER_MEDIUM | string::MARKER_LARGE => {
                            Ok(Message::Init(Init::try_from(remaining_bytes_arc)?))
                        }
                        // Else, must be a metadata map
                        _ => Ok(Message::Hello(Hello::try_from(remaining_bytes_arc)?)),
                    }
                }
                run::SIGNATURE => Ok(Message::Run(Run::try_from(remaining_bytes_arc)?)),
                discard_all::SIGNATURE => Ok(Message::DiscardAll),
                pull_all::SIGNATURE => Ok(Message::PullAll),
                ack_failure::SIGNATURE => Ok(Message::AckFailure),
                reset::SIGNATURE => Ok(Message::Reset),
                record::SIGNATURE => Ok(Message::Record(Record::try_from(remaining_bytes_arc)?)),
                success::SIGNATURE => Ok(Message::Success(Success::try_from(remaining_bytes_arc)?)),
                failure::SIGNATURE => Ok(Message::Failure(Failure::try_from(remaining_bytes_arc)?)),
                ignored::SIGNATURE => Ok(Message::Ignored),
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
