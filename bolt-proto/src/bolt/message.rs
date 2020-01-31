use std::convert::{TryFrom, TryInto};
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use tokio::io::BufStream;
use tokio::prelude::*;

pub use ack_failure::AckFailure;
pub(crate) use chunk::Chunk;
pub use discard_all::DiscardAll;
pub use failure_::Failure;
pub use ignored::Ignored;
pub use init::Init;
pub(crate) use message_bytes::MessageBytes;
pub use pull_all::PullAll;
pub use record::Record;
pub use reset::Reset;
pub use run::Run;
pub use success::Success;

use crate::bolt::structure::get_signature_from_bytes;
use crate::error::*;
use crate::{native, Deserialize, Marker, Serialize, Signature};

mod ack_failure;
mod chunk;
mod discard_all;
mod failure_;
mod ignored;
mod init;
mod message_bytes;
mod pull_all;
mod record;
mod reset;
mod run;
mod success;

// This is what's used in the protocol spec, but it could technically be any size.
const CHUNK_SIZE: usize = 16; // TODO: Make this configurable

#[derive(Debug, Clone)]
pub enum Message {
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
}

impl Message {
    pub async fn from_stream<T: Unpin + AsyncRead + AsyncWrite>(
        buf_stream: &mut BufStream<T>,
    ) -> Result<Message> {
        Message::try_from(MessageBytes::from_stream(buf_stream).await?)
    }
}

impl From<native::message::Init> for Message {
    fn from(message: native::message::Init) -> Self {
        Message::Init(Init::from(message))
    }
}

impl From<native::message::Run> for Message {
    fn from(message: native::message::Run) -> Self {
        Message::Run(Run::from(message))
    }
}

impl From<native::message::Record> for Message {
    fn from(message: native::message::Record) -> Self {
        Message::Record(Record::from(message))
    }
}

impl From<native::message::Success> for Message {
    fn from(message: native::message::Success) -> Self {
        Message::Success(Success::from(message))
    }
}

impl From<native::message::Failure> for Message {
    fn from(message: native::message::Failure) -> Self {
        Message::Failure(Failure::from(message))
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
        let result: Result<Message> = catch_unwind(move || {
            let signature = get_signature_from_bytes(&mut message_bytes)?;
            let remaining_bytes_arc =
                Arc::new(Mutex::new(message_bytes.split_to(message_bytes.len())));

            match signature {
                init::SIGNATURE => Ok(Message::Init(Init::try_from(remaining_bytes_arc)?)),
                run::SIGNATURE => Ok(Message::Run(Run::try_from(remaining_bytes_arc)?)),
                discard_all::SIGNATURE => Ok(Message::DiscardAll),
                pull_all::SIGNATURE => Ok(Message::PullAll),
                ack_failure::SIGNATURE => Ok(Message::AckFailure),
                reset::SIGNATURE => Ok(Message::Reset),
                record::SIGNATURE => Ok(Message::Record(Record::try_from(remaining_bytes_arc)?)),
                success::SIGNATURE => Ok(Message::Success(Success::try_from(remaining_bytes_arc)?)),
                failure_::SIGNATURE => {
                    Ok(Message::Failure(Failure::try_from(remaining_bytes_arc)?))
                }
                ignored::SIGNATURE => Ok(Message::Ignored),
                _ => {
                    Err(DeserializeError(format!("Invalid signature byte: {:x}", signature)).into())
                }
            }
        })
        .map_err(|_| DeserializeError("Panicked during deserialization".to_string()))?;

        Ok(result.map_err(|err: Error| {
            DeserializeError(format!("Error creating Message from Bytes: {}", err))
        })?)
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
