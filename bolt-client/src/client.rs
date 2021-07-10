// Much of the documentation comments for message-related instance methods in Client and
// its submodules are copied from the descriptions given by Neo Technology, Inc. on
// https://boltprotocol.org/v1/, with minor modifications.
//
// The aforementioned comments are thus licensed under the Creative Commons
// Attribution-ShareAlike 3.0 Unported License. To view a copy of this license, visit
// http://creativecommons.org/licenses/by-sa/3.0/ or send a letter to Creative Commons,
// PO Box 1866, Mountain View, CA 94042, USA.

use std::collections::VecDeque;

use bytes::*;
use futures_util::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use bolt_proto::{error::Error as ProtocolError, Message, ServerState, ServerState::*};

use crate::error::{CommunicationError, CommunicationResult, ConnectionError, ConnectionResult};

mod v1;
mod v2;
mod v3;
mod v4;
mod v4_1;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];

/// An asynchronous client for Bolt servers.
#[derive(Debug)]
pub struct Client<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
    version: u32,
    server_state: ServerState,
    sent_queue: VecDeque<Message>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    /// Attempt to create a new client from an asynchronous stream. A handshake will be
    /// performed with the provided protocol versions, and, if this succeeds, a Client will be
    /// returned.
    pub async fn new(mut stream: S, preferred_versions: &[u32; 4]) -> ConnectionResult<Self> {
        let mut preferred_versions_bytes = BytesMut::with_capacity(16);
        preferred_versions
            .iter()
            .for_each(|&v| preferred_versions_bytes.put_u32(v));
        stream.write_all(&PREAMBLE).await?;
        stream.write_all(&preferred_versions_bytes).await?;
        stream.flush().await?;

        let mut u32_bytes = [0, 0, 0, 0];
        stream.read_exact(&mut u32_bytes).await?;
        let version = u32::from_be_bytes(u32_bytes);
        if preferred_versions.contains(&version) && version > 0 {
            Ok(Self {
                stream,
                version,
                server_state: Connected,
                sent_queue: Default::default(),
            })
        } else {
            Err(ConnectionError::HandshakeFailed(*preferred_versions))
        }
    }

    /// Get the current version of this client.
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn server_state(&self) -> ServerState {
        self.server_state
    }

    pub(crate) async fn read_message(&mut self) -> CommunicationResult<Message> {
        let message = Message::from_stream(&mut self.stream)
            .await
            .map_err(ProtocolError::from)?;

        #[cfg(test)]
        println!("<<< {:?}\n", message);

        // TODO: Use or-patterns where possible
        match (self.server_state, self.sent_queue.pop_front(), message) {
            // CONNECTED
            (Connected, Some(Message::Init(_)), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Connected, Some(Message::Init(_)), Message::Failure(failure)) => {
                self.server_state = Defunct;
                Ok(Message::Failure(failure))
            }
            (Connected, Some(Message::Hello(_)), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Connected, Some(Message::Hello(_)), Message::Failure(failure)) => {
                self.server_state = Defunct;
                Ok(Message::Failure(failure))
            }

            // READY
            (Ready, Some(Message::Run(_)), Message::Success(success)) => {
                self.server_state = Streaming;
                Ok(Message::Success(success))
            }
            (Ready, Some(Message::Run(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (Ready, Some(Message::RunWithMetadata(_)), Message::Success(success)) => {
                self.server_state = Streaming;
                Ok(Message::Success(success))
            }
            (Ready, Some(Message::RunWithMetadata(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (Ready, Some(Message::Begin(_)), Message::Success(success)) => {
                self.server_state = TxReady;
                Ok(Message::Success(success))
            }
            (Ready, Some(Message::Begin(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }

            // STREAMING
            (Streaming, Some(Message::PullAll), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Streaming, Some(Message::PullAll), Message::Record(record)) => {
                self.server_state = Streaming;
                // Put the PULL_ALL message back so we can keep consuming records
                self.sent_queue.push_front(Message::PullAll);
                Ok(Message::Record(record))
            }
            (Streaming, Some(Message::PullAll), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (Streaming, Some(Message::Pull(_)), Message::Success(success)) => {
                // TODO: Check has_more field
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Streaming, Some(Message::Pull(pull)), Message::Record(record)) => {
                self.server_state = Streaming;
                // Put the PULL message back so we can keep consuming records
                self.sent_queue.push_front(Message::Pull(pull));
                Ok(Message::Record(record))
            }
            (Streaming, Some(Message::Pull(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (Streaming, Some(Message::DiscardAll), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Streaming, Some(Message::DiscardAll), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (Streaming, Some(Message::Discard(_)), Message::Success(success)) => {
                // TODO: Check has_more field
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Streaming, Some(Message::Discard(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }

            // TX_READY
            (TxReady, Some(Message::RunWithMetadata(_)), Message::Success(success)) => {
                self.server_state = TxStreaming;
                Ok(Message::Success(success))
            }
            (TxReady, Some(Message::RunWithMetadata(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (TxReady, Some(Message::Commit), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (TxReady, Some(Message::Commit), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (TxReady, Some(Message::Rollback), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (TxReady, Some(Message::Rollback), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }

            // TX_STREAMING
            (TxStreaming, Some(Message::RunWithMetadata(_)), Message::Success(success)) => {
                self.server_state = TxStreaming;
                Ok(Message::Success(success))
            }
            (TxStreaming, Some(Message::RunWithMetadata(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (TxStreaming, Some(Message::PullAll), Message::Success(success)) => {
                self.server_state = TxReady;
                Ok(Message::Success(success))
            }
            (TxStreaming, Some(Message::PullAll), Message::Record(record)) => {
                self.server_state = TxStreaming;
                // Put the PULL_ALL message back so we can keep consuming records
                self.sent_queue.push_front(Message::PullAll);
                Ok(Message::Record(record))
            }
            (TxStreaming, Some(Message::PullAll), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (TxStreaming, Some(Message::Pull(_)), Message::Success(success)) => {
                // TODO: Check has_more field
                self.server_state = TxReady;
                Ok(Message::Success(success))
            }
            (TxStreaming, Some(Message::Pull(pull)), Message::Record(record)) => {
                self.server_state = TxStreaming;
                // Put the PULL message back so we can keep consuming records
                self.sent_queue.push_front(Message::Pull(pull));
                Ok(Message::Record(record))
            }
            (TxStreaming, Some(Message::Pull(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (TxStreaming, Some(Message::DiscardAll), Message::Success(success)) => {
                self.server_state = TxReady;
                Ok(Message::Success(success))
            }
            (TxStreaming, Some(Message::DiscardAll), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }
            (TxStreaming, Some(Message::Discard(_)), Message::Success(success)) => {
                // TODO: Check has_more field
                self.server_state = TxReady;
                Ok(Message::Success(success))
            }
            (TxStreaming, Some(Message::Discard(_)), Message::Failure(failure)) => {
                self.server_state = Failed;
                Ok(Message::Failure(failure))
            }

            // FAILED
            (Failed, Some(Message::Run(_)), Message::Ignored) => {
                self.server_state = Failed;
                Ok(Message::Ignored)
            }
            (Failed, Some(Message::RunWithMetadata(_)), Message::Ignored) => {
                self.server_state = Failed;
                Ok(Message::Ignored)
            }
            (Failed, Some(Message::PullAll), Message::Ignored) => {
                self.server_state = Failed;
                Ok(Message::Ignored)
            }
            (Failed, Some(Message::Pull(_)), Message::Ignored) => {
                self.server_state = Failed;
                Ok(Message::Ignored)
            }
            (Failed, Some(Message::DiscardAll), Message::Ignored) => {
                self.server_state = Failed;
                Ok(Message::Ignored)
            }
            (Failed, Some(Message::Discard(_)), Message::Ignored) => {
                self.server_state = Failed;
                Ok(Message::Ignored)
            }
            (Failed, Some(Message::AckFailure), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Failed, Some(Message::AckFailure), Message::Failure(failure)) => {
                self.server_state = Defunct;
                Ok(Message::Failure(failure))
            }

            // INTERRUPTED
            (Interrupted, Some(Message::Run(_)), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::RunWithMetadata(_)), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::PullAll), Message::Record(_)) => {
                self.server_state = Interrupted;
                // Put the PULL_ALL message back so we can keep consuming records
                self.sent_queue.push_front(Message::PullAll);
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::PullAll), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Pull(pull)), Message::Record(_)) => {
                self.server_state = Interrupted;
                // Put the PULL message back so we can keep consuming records
                self.sent_queue.push_front(Message::Pull(pull));
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Pull(_)), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::DiscardAll), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Discard(_)), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Begin(_)), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Commit), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Rollback), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::AckFailure), _) => {
                self.server_state = Interrupted;
                Ok(Message::Ignored)
            }
            (Interrupted, Some(Message::Reset), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
            }
            (Interrupted, Some(Message::Reset), Message::Failure(failure)) => {
                self.server_state = Defunct;
                Ok(Message::Failure(failure))
            }

            (state, request, response) => {
                self.server_state = Defunct;
                return Err(CommunicationError::InvalidResponse {
                    state,
                    request,
                    response,
                });
            }
        }
    }

    // TODO: Handle immediate state changes
    pub(crate) async fn send_message(&mut self, message: Message) -> CommunicationResult<()> {
        // TODO: Use or-patterns where possible
        match (self.server_state, &message) {
            (Connected, Message::Init(_)) => {}
            (Connected, Message::Hello(_)) => {}
            (Ready, Message::Run(_)) => {}
            (Ready, Message::RunWithMetadata(_)) => {}
            (Ready, Message::Begin(_)) => {}
            (Ready, Message::Reset) => {}
            (Ready, Message::Goodbye) => {}
            (Streaming, Message::PullAll) => {}
            (Streaming, Message::Pull(_)) => {}
            (Streaming, Message::DiscardAll) => {}
            (Streaming, Message::Discard(_)) => {}
            (Streaming, Message::Reset) => {}
            (Streaming, Message::Goodbye) => {}
            (TxReady, Message::RunWithMetadata(_)) => {}
            (TxReady, Message::Commit) => {}
            (TxReady, Message::Rollback) => {}
            (TxReady, Message::Reset) => {}
            (TxReady, Message::Goodbye) => {}
            (TxStreaming, Message::RunWithMetadata(_)) => {}
            (TxStreaming, Message::PullAll) => {}
            (TxStreaming, Message::Pull(_)) => {}
            (TxStreaming, Message::DiscardAll) => {}
            (TxStreaming, Message::Discard(_)) => {}
            (TxStreaming, Message::Reset) => {}
            (TxStreaming, Message::Goodbye) => {}
            (Failed, Message::Run(_)) => {}
            (Failed, Message::RunWithMetadata(_)) => {}
            (Failed, Message::PullAll) => {}
            (Failed, Message::Pull(_)) => {}
            (Failed, Message::DiscardAll) => {}
            (Failed, Message::Discard(_)) => {}
            (Failed, Message::AckFailure) => {}
            (Failed, Message::Reset) => {}
            (Failed, Message::Goodbye) => {}
            (Interrupted, Message::Run(_)) => {}
            (Interrupted, Message::RunWithMetadata(_)) => {}
            (Interrupted, Message::PullAll) => {}
            (Interrupted, Message::Pull(_)) => {}
            (Interrupted, Message::DiscardAll) => {}
            (Interrupted, Message::Discard(_)) => {}
            (Interrupted, Message::Begin(_)) => {}
            (Interrupted, Message::Commit) => {}
            (Interrupted, Message::Rollback) => {}
            (Interrupted, Message::Reset) => {}
            (Interrupted, Message::Goodbye) => {}
            (state, message) => {
                self.server_state = Defunct;
                return Err(CommunicationError::InvalidState {
                    state,
                    message: message.clone(),
                });
            }
        }

        #[cfg(test)]
        println!(">>> {:?}", message);

        let chunks = message.clone().into_chunks().map_err(ProtocolError::from)?;

        for chunk in chunks {
            self.stream.write_all(&chunk).await?;
        }
        self.stream.flush().await?;

        // Immediate state changes
        match message {
            Message::Reset => self.server_state = Interrupted,
            Message::Goodbye => self.server_state = Disconnected,
            _ => {}
        }

        self.sent_queue.push_back(message);
        Ok(())
    }

    /// Send multiple messages to the server without waiting for a response. Returns a
    /// [`Vec`] containing the server's response messages for each of the sent messages,
    /// in the order they were provided.
    ///
    /// # Description
    /// The client is not required to wait for a response before sending more messages.
    /// Sending multiple messages together like this is called pipelining. For performance
    /// reasons, it is recommended that clients use pipelining as much as possible.
    /// Through pipelining, multiple messages can be transmitted together in the same
    /// network package, significantly reducing latency and increasing throughput.
    ///
    /// A common technique is to buffer outgoing messages on the client until the last
    /// possible moment, such as when a commit is issued or a result is read by the
    /// application, and then sending all messages in the buffer together.
    ///
    /// # Failure Handling
    /// Because the protocol leverages pipelining, the client and the server need to agree
    /// on what happens when a failure occurs, otherwise messages that were sent assuming
    /// no failure would occur might have unintended effects.
    ///
    /// When requests fail on the server, the server will send the client a `FAILURE`
    /// message. The client must acknowledge the `FAILURE` message by sending a `RESET`
    /// (Bolt v3+) or `ACK_FAILURE` (Bolt v1-2) message to the server. Until the server
    /// receives the `RESET`/`ACK_FAILURE` message, it will send an `IGNORED` message in
    /// response to any other message from the client, including messages that were sent
    /// in a pipeline.
    pub async fn pipeline(&mut self, messages: Vec<Message>) -> CommunicationResult<Vec<Message>> {
        // This Vec is too small if we're expecting some RECORD messages, so there's no "good" size
        let mut responses = Vec::with_capacity(messages.len());

        for message in &messages {
            #[cfg(test)]
            println!(">>> {:?}", message);

            let chunks = message.clone().into_chunks().map_err(ProtocolError::from)?;

            for chunk in chunks {
                self.stream.write_all(&chunk).await?;
            }

            // Immediate state changes
            match message {
                Message::Reset => self.server_state = Interrupted,
                Message::Goodbye => self.server_state = Disconnected,
                _ => {}
            }
        }
        self.stream.flush().await?;
        self.sent_queue.extend(messages);

        // FIXME: If a RESET was sent, some RECORD messages may be replaced with IGNORED, and we
        //        will not account for them here, meaning that some messages will be left in the
        //        socket buffer at the end. Make sure that all messages are consumed here.
        for _ in 0..responses.capacity() {
            let mut response = self.read_message().await?;
            while let Message::Record(_) = response {
                responses.push(response);
                response = self.read_message().await?;
            }
            responses.push(response);
        }
        Ok(responses)
    }
}
