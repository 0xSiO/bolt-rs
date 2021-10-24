// Much of the documentation comments in this module are copied from the descriptions on
// https://7687.org, with minor modifications.
//
// Original copyright and license information for these descriptions:
// Copyright © 2002-2020 Neo4j, Inc.
// CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/)
//
// The aforementioned documentation comments are thus licensed under CC BY-SA 4.0.

use std::{collections::VecDeque, convert::TryInto, io};

use bytes::*;
use futures_util::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use bolt_client_macros::*;
use bolt_proto::{
    error::Error as ProtocolError, message::*, version::*, Message, ServerState, ServerState::*,
    Value,
};

use crate::{
    error::{CommunicationError, CommunicationResult, ConnectionError, ConnectionResult},
    Metadata, Params, RoutingContext,
};

mod v1;
mod v2;
mod v3;
mod v4;
mod v4_1;
mod v4_2;
mod v4_3;

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
        // TODO: Update to handle ranges
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

    /// Get the current server state for this client.
    pub fn server_state(&self) -> ServerState {
        self.server_state
    }

    pub(crate) async fn read_message(&mut self) -> CommunicationResult<Message> {
        let message = Message::from_stream(&mut self.stream)
            .await
            .map_err(ProtocolError::from)?;

        #[cfg(test)]
        println!("<<< {:?}\n", message);

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
            (Ready, Some(Message::Route(_)), Message::Success(success)) => {
                self.server_state = Ready;
                Ok(Message::Success(success))
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
                self.server_state = match success.metadata().get("has_more") {
                    Some(&Value::Boolean(true)) => Streaming,
                    _ => Ready,
                };
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
                self.server_state = match success.metadata().get("has_more") {
                    Some(&Value::Boolean(true)) => Streaming,
                    _ => Ready,
                };
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
                self.server_state = match success.metadata().get("has_more") {
                    Some(&Value::Boolean(true)) => TxStreaming,
                    _ => TxReady, // TODO: Or TxStreaming, if there are other streams open?
                };
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
                self.server_state = match success.metadata().get("has_more") {
                    Some(&Value::Boolean(true)) => TxStreaming,
                    _ => TxReady, // TODO: Or TxStreaming, if there are other streams open?
                };
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
                Err(CommunicationError::InvalidResponse {
                    state,
                    request,
                    response,
                })
            }
        }
    }

    pub(crate) async fn send_message(&mut self, message: Message) -> CommunicationResult<()> {
        match (self.server_state, &message) {
            (Connected, Message::Init(_)) => {}
            (Connected, Message::Hello(_)) => {}
            (Ready, Message::Run(_)) => {}
            (Ready, Message::RunWithMetadata(_)) => {}
            (Ready, Message::Begin(_)) => {}
            (Ready, Message::Route(_)) => {}
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
            (Interrupted, Message::AckFailure) => {}
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

    /// Send a [`HELLO`](Message::Hello) (or [`INIT`](Message::Init)) message to the server.
    /// _(Sends `INIT` for Bolt v1 - v2, and `HELLO` for Bolt v3+.)_
    ///
    /// # Description
    /// The `HELLO` message requests the connection to be authorized for use with the remote
    /// database. Clients should send a `HELLO` message to the server immediately after connection
    /// and process the response before using that connection in any other way.
    ///
    /// The server must be in the [`Connected`](bolt_proto::ServerState::Connected) state to be
    /// able to process a `HELLO` message. For any other states, receipt of a `HELLO` message is
    /// considered a protocol violation and leads to connection closure.
    ///
    /// If authentication fails, the server will respond with a [`FAILURE`](Message::Failure)
    /// message and immediately close the connection. Clients wishing to retry initialization
    /// should establish a new connection.
    ///
    /// # Fields
    /// `metadata` should contain at least two entries:
    /// - `user_agent`, which should conform to the format `"Name/Version"`, for example
    ///   `"Example/1.0.0"` (see
    ///   [here](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent)).
    /// - `scheme` is the authentication scheme. Predefined schemes are `"none"`, `"basic"`, or
    ///   `"kerberos"`.
    ///
    /// If using Bolt v4.3 or later, the following additional `metadata` entries can be specified:
    /// - `routing`, a map which should contain routing context information as well as an `address`
    ///   field indicating to which address the client should initially connect. Leaving this
    ///   unspecified indicates that the server should not carry out any routing.
    ///   _(Bolt v4.3+ only.)_
    ///
    /// Further entries in `metadata` are passed to the implementation of the chosen
    /// authentication scheme. Their names, types, and defaults depend on that choice. For
    /// example, the scheme `"basic"` requires `metadata` to contain the username and password in
    /// the form `{"principal": "<username>", "credentials": "<password>"}`.
    ///
    /// # Response
    /// - [`Message::Success`] - initialization has completed successfully and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server may include
    ///   metadata that describes details of the server environment and/or the connection. The
    ///   following fields are defined for inclusion in the `SUCCESS` metadata:
    ///   - `server`, the server agent string (e.g. `"Neo4j/4.3.0"`)
    ///   - `connection_id`, a unique identifier for the connection (e.g. `"bolt-61"`)
    ///     _(Bolt v3+ only.)_
    ///   - `hints`, a map of configuration hints (e.g. `{"connection.recv_timeout_seconds": 120}`)
    ///     These hints may be interpreted or ignored by drivers at their own discretion in order
    ///     to augment operations where applicable. Hints remain valid throughout the lifetime of a
    ///     given connection and cannot be changed. As such, newly established connections may
    ///     observe different hints as the server configuration is adjusted.
    ///     _(Bolt v4.3+ only.)_
    /// - [`Message::Failure`] - initialization has failed and the server has entered the
    ///   [`Defunct`](bolt_proto::ServerState::Defunct) state. The server may choose to include
    ///   metadata describing the nature of the failure but will immediately close the connection
    ///   after the failure has been sent.
    #[bolt_version(1, 2, 3, 4, 4.1, 4.2, 4.3)]
    pub async fn hello(&mut self, mut metadata: Metadata) -> CommunicationResult<Message> {
        let message = match self.version() {
            V1_0 | V2_0 => {
                let user_agent: String = metadata
                    .value
                    .remove("user_agent")
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "missing user_agent")
                    })?
                    .try_into()
                    .map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "user_agent must be a string")
                    })?;
                let auth_token = metadata.value;

                Message::Init(Init::new(user_agent, auth_token))
            }
            _ => Message::Hello(Hello::new(metadata.value)),
        };

        self.send_message(message).await?;
        self.read_message().await
    }

    // TODO: Add additional allowed server states for `RUN`
    /// Send a [`RUN`](Message::Run) message to the server.
    /// _(Bolt v1+. For Bolt v1 - v2, the `metadata` parameter is ignored.)_
    ///
    /// # Description
    /// A `RUN` message submits a new query for execution, the result of which will be consumed by
    /// a subsequent message, such as [`PULL_ALL`](Message::PullAll).
    ///
    /// The server must be in the [`Ready`](bolt_proto::ServerState::Ready) state to be able to
    /// successfully process a `RUN` request. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `RUN` request will be
    /// considered a protocol violation and will lead to connection closure.
    ///
    /// # Fields
    /// - `query` contains a database query or remote procedure call.
    /// - `parameters` contains variable fields for `query`.
    ///
    /// If using Bolt v3 or later, the following `metadata` entries can be specified:
    /// - `bookmarks`, a list of strings containing some kind of bookmark identification, e.g
    ///   `["bkmk-transaction:1", "bkmk-transaction:2"]`. Default is `[]`.
    /// - `tx_timeout`, an integer specifying a transaction timeout in milliseconds. Default is the
    ///   server-side configured timeout.
    /// - `tx_metadata`, a map containing some metadata information, mainly used for logging.
    /// - `mode`, a string which specifies what kind of server should be used for this
    ///   transaction. For write access, use `"w"` and for read access use `"r"`. Default is `"w"`.
    /// - `db`, a string containing the name of the database where the transaction should take
    ///   place. [`null`](bolt_proto::Value::Null) and `""` denote the server-side configured
    ///   default database. Default is `null`. _(Bolt v4+ only.)_
    ///
    /// # Response
    /// - [`Message::Success`] - the request has been successfully received and the server has
    ///   entered the [`Streaming`](bolt_proto::ServerState::Streaming) state. Clients should not
    ///   consider a `SUCCESS` response to indicate completion of the execution of the query,
    ///   merely acceptance of it. The server may attach metadata to the message to provide header
    ///   detail for the results that follow. The following fields are defined for inclusion in the
    ///   metadata:
    ///   - `fields`, the fields included in the result (e.g. `["name", "age"]`)
    ///   - `result_available_after`, the time in milliseconds after which the first record in the
    ///     result stream is available. _(Bolt v1 - v2 only.)_
    ///   - `t_first`, supercedes `result_available_after`. _(Bolt v3+ only.)_
    ///   - `qid`, an integer that specifies the server-assigned query ID. This is sent for
    ///     queries submitted within an explicit transaction. _(Bolt v4+ only.)_
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully or is invalid, and
    ///   the server has entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server
    ///   may attach metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(1, 2, 3, 4, 4.1, 4.2, 4.3)]
    pub async fn run(
        &mut self,
        query: impl Into<String>,
        parameters: Option<Params>,
        metadata: Option<Metadata>,
    ) -> CommunicationResult<Message> {
        let message = match self.version() {
            V1_0 | V2_0 => {
                Message::Run(Run::new(query.into(), parameters.unwrap_or_default().value))
            }
            _ => Message::RunWithMetadata(RunWithMetadata::new(
                query.into(),
                parameters.unwrap_or_default().value,
                metadata.unwrap_or_default().value,
            )),
        };

        self.send_message(message).await?;
        self.read_message().await
    }

    /// Send a [`DISCARD`](Message::Discard) (or [`DISCARD_ALL`](Message::DiscardAll)) message to
    /// the server.
    /// _(Sends a `DISCARD_ALL` for Bolt v1 - v3, and `DISCARD` for Bold v4+. For Bolt v1 - v3,
    /// the `metadata` parameter is ignored.)_
    ///
    /// # Description
    /// The `DISCARD` message issues a request to discard the outstanding result and return to the
    /// [`Ready`](bolt_proto::ServerState::Ready) state. A receiving server will not abort the
    /// request but continue to process it without streaming any detail messages to the client.
    ///
    /// The server must be in the [`Streaming`](bolt_proto::ServerState::Streaming) or
    /// [`TxStreaming`](bolt_proto::ServerState::TxStreaming) state to be able to successfully
    /// process a `DISCARD` request. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) state or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `DISCARD` request
    /// will be considered a protocol violation and will lead to connection closure.
    ///
    /// # Fields
    /// For Bolt v4+, additional metadata is passed along with this message:
    /// - `n` is an integer specifying how many records to discard. `-1` will discard all records.
    ///   `n` has no default and must be present.
    /// - `qid` is an integer that specifies for which statement the `DISCARD` operation should be
    ///   carried out within an explicit transaction. `-1` is the default, which denotes the last
    ///   executed statement.
    ///
    /// # Response
    /// - [`Message::Success`] - results have been successfully discarded and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server may attach
    ///   metadata to the message to provide footer detail for the discarded results.
    ///   The following fields are defined for inclusion in the metadata:
    ///   - `type`, the type of query: read-only (`"r"`), write-only (`"w"`), read-write (`"rw"`),
    ///     or schema (`"s"`)
    ///   - `result_consumed_after`, the time in milliseconds after which the last record in the
    ///     result stream is consumed. _(Bolt v1 - v2 only.)_
    ///   - `t_last`, supercedes `result_consumed_after`. _(Bolt v3+ only.)_
    ///   - `bookmark` (e.g. `"bookmark:1234"`). _(Bolt v3+ only.)_
    ///   - `db`, a string containing the name of the database where the query was executed.
    ///     _(Bolt v4+ only.)_
    ///   - `has_more`, a boolean indicating whether there are still records left in the result
    ///     stream. Default is `false`. _(Bolt v4+ only.)_
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server may attach
    ///   metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(1, 2, 3, 4, 4.1, 4.2, 4.3)]
    pub async fn discard(&mut self, metadata: Option<Metadata>) -> CommunicationResult<Message> {
        let message = match self.version() {
            V1_0 | V2_0 | V3_0 => Message::DiscardAll,
            _ => Message::Discard(Discard::new(metadata.unwrap_or_default().value)),
        };
        self.send_message(message).await?;
        self.read_message().await
    }

    /// Send a [`PULL`](Message::Pull) (or [`PULL_ALL`](Message::PullAll)) message to the server.
    /// _(Sends `PULL_ALL` for Bolt v1 - v3, and `PULL` for Bolt v4+. For Bolt v1 - v3, the
    /// `metadata` parameter is ignored.)_
    ///
    /// # Description
    /// The `PULL` message issues a request to stream outstanding results back to the client,
    /// before returning to the [`Ready`](bolt_proto::ServerState::Ready) state.
    ///
    /// Result details consist of zero or more [`RECORD`](Message::Record) messages and a summary
    /// message. Each record carries with it a list of values which form the data content of the
    /// record. The order of the values within that list should be meaningful to the client,
    /// perhaps based on a requested ordering for that result, but no guarantees are made around
    /// the order of records within the result. A record should only be considered valid if
    /// accompanied by a [`SUCCESS`](Message::Success) summary message.
    ///
    /// The server must be in the [`Streaming`](bolt_proto::ServerState::Streaming) or
    /// [`TxStreaming`](bolt_proto::ServerState::TxStreaming) state to be able to successfully
    /// process a `PULL` request. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) state or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `PULL` request will
    /// be considered a protocol violation and will lead to connection closure.
    ///
    /// # Fields
    /// For Bolt v4+, additional metadata is passed along with this message:
    /// - `n` is an integer specifying how many records to fetch. `-1` will fetch all records. `n`
    ///   has no default and must be present.
    /// - `qid` is an integer that specifies for which statement the `PULL` operation should be
    ///   carried out within an explicit transaction. `-1` is the default, which denotes the last
    ///   executed statement.
    ///
    /// # Response
    /// - `(_, `[`Message::Success`]`)` - results have been successfully pulled and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server may attach
    ///   metadata to the `SUCCESS` message to provide footer detail for the results. The
    ///   following fields are defined for inclusion in the metadata:
    ///   - `type`, the type of query: read-only (`"r"`), write-only (`"w"`), read-write (`"rw"`),
    ///     or schema (`"s"`)
    ///   - `result_consumed_after`, the time in milliseconds after which the last record in the
    ///     result stream is consumed. _(Bolt v1 - v2 only.)_
    ///   - `t_last`, supercedes `result_consumed_after`. _(Bolt v3+ only.)_
    ///   - `bookmark` (e.g. `"bookmark:1234"`). _(Bolt v3+ only.)_
    ///   - `stats`, a map containing counter information, such as DB hits, etc. _(Bolt v3+ only.)_
    ///   - `plan`, a map containing the query plan result. _(Bolt v3+ only.)_
    ///   - `profile`, a map containing the query profile result. _(Bolt v3+ only.)_
    ///   - `notifications`: a map containing any notifications generated during execution of the
    ///     query. _(Bolt v3+ only.)_
    ///   - `db`, a string containing the name of the database where the query was executed.
    ///     _(Bolt v4+ only.)_
    ///   - `has_more`, a boolean indicating whether there are still records left in the result
    ///     stream. Default is `false`. _(Bolt v4+ only.)_
    /// - `(_, `[`Message::Ignored`]`)` - the server is in the
    ///   [`Failed`](bolt_proto::ServerState::Failed) or
    ///   [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - `(_, `[`Message::Failure`]`)` - the request could not be processed
    ///   successfully and the server has entered the [`Failed`](bolt_proto::ServerState::Failed)
    ///   state. The server may attach metadata to the message to provide more detail on the
    ///   nature of the failure. Failure may occur at any time during result streaming, so any
    ///   records returned in the response should be considered invalid.
    #[bolt_version(1, 2, 3, 4, 4.1, 4.2, 4.3)]
    pub async fn pull(
        &mut self,
        metadata: Option<Metadata>,
    ) -> CommunicationResult<(Vec<Record>, Message)> {
        match self.version() {
            V1_0 | V2_0 | V3_0 => self.send_message(Message::PullAll).await?,
            _ => {
                self.send_message(Message::Pull(Pull::new(metadata.unwrap_or_default().value)))
                    .await?
            }
        }
        let mut records = vec![];
        loop {
            match self.read_message().await? {
                Message::Record(record) => records.push(record),
                Message::Success(success) => return Ok((records, Message::Success(success))),
                Message::Failure(failure) => return Ok((records, Message::Failure(failure))),
                Message::Ignored => return Ok((vec![], Message::Ignored)),
                _ => unreachable!(),
            }
        }
    }

    /// Send an [`ACK_FAILURE`](Message::AckFailure) message to the server.
    /// _(Bolt v1 - v2 only. For Bolt v3+, see [`Client::reset`].)_
    ///
    /// # Description
    /// `ACK_FAILURE` signals to the server that the client has acknowledged a previous failure and
    /// should return to the [`Ready`](bolt_proto::ServerState::Ready) state.
    ///
    /// The server must be in the [`Failed`](bolt_proto::ServerState::Failed) state to be able to
    /// successfully process an `ACK_FAILURE` request. For any other states, receipt of an
    /// `ACK_FAILURE` request will be considered a protocol violation and will lead to connection
    /// closure.
    ///
    /// # Response
    /// - [`Message::Success`] - failure has been successfully acknowledged and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server may attach
    ///   metadata to the `SUCCESS` message.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Defunct`](bolt_proto::ServerState::Defunct) state. The server may choose to
    ///   include metadata describing the nature of the failure but will immediately close the
    ///   connection after the failure has been sent.
    #[bolt_version(1, 2)]
    pub async fn ack_failure(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::AckFailure).await?;
        self.read_message().await
    }

    /// Send a [`RESET`](Message::Reset) message to the server.
    /// _(Bolt v1+. For Bolt v1 - v2, see [`Client::ack_failure`] for just clearing the
    /// [`Failed`](bolt_proto::ServerState::Failed) state.)_
    ///
    /// # Description
    /// The `RESET` message requests that the connection be set back to its initial state, as if
    /// initialization had just been successfully completed. The `RESET` message is unique in that
    /// it on arrival at the server, it jumps ahead in the message queue, stopping any unit of work
    /// that happens to be executing. All the queued messages originally in front of the `RESET`
    /// message will then be [`IGNORED`](Message::Ignored) until the `RESET` position is reached,
    /// at which point the server will be ready for a new session.
    ///
    /// Specifically, `RESET` will:
    /// - force any currently processing message to abort with [`IGNORED`](Message::Ignored)
    /// - force any pending messages that have not yet started processing to be
    ///   [`IGNORED`](Message::Ignored)
    /// - clear any outstanding [`Failed`](bolt_proto::ServerState::Failed) state
    /// - dispose of any outstanding result records
    /// - cancel the current transaction, if any
    ///
    /// # Response
    /// - [`Message::Success`] - the session has been successfully reset and the server has entered
    ///   the [`Ready`](bolt_proto::ServerState::Ready) state.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Defunct`](bolt_proto::ServerState::Defunct) state. The server may choose to
    ///   include metadata describing the nature of the failure but will immediately close the
    ///   connection after the failure has been sent.
    #[bolt_version(1, 2, 3, 4, 4.1, 4.2, 4.3)]
    pub async fn reset(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::Reset).await?;
        loop {
            match self.read_message().await? {
                Message::Success(success) => return Ok(Message::Success(success)),
                Message::Failure(failure) => return Ok(Message::Failure(failure)),
                Message::Ignored => {}
                _ => unreachable!(),
            }
        }
    }

    /// Send a [`GOODBYE`](Message::Goodbye) message to the server.
    /// _(Bolt v3+ only.)_
    ///
    /// # Description
    /// The `GOODBYE` message notifies the server that the connection is terminating gracefully. On
    /// receipt of this message, the server will immediately shut down the socket on its side
    /// without sending a response. A client may shut down the socket at any time after sending the
    /// `GOODBYE` message. This message interrupts the server's current work, if any.
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn goodbye(&mut self) -> CommunicationResult<()> {
        self.send_message(Message::Goodbye).await?;
        self.server_state = Defunct;
        Ok(self.stream.close().await?)
    }

    /// Send a [`BEGIN`](Message::Begin) message to the server.
    /// _(Bolt v3+ only.)_
    ///
    /// # Description
    /// The `BEGIN` message starts a new explicit transaction and transitions the server to the
    /// [`TxReady`](bolt_proto::ServerState::TxReady) state. The explicit transaction is closed
    /// with either the [`COMMIT`](Message::Commit) message or [`ROLLBACK`](Message::Rollback)
    /// message.
    ///
    /// The server must be in the [`Ready`](bolt_proto::ServerState::Ready) state to be able to
    /// successfully process a `BEGIN` request. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `BEGIN` request will be
    /// considered a protocol violation and will lead to connection closure.
    ///
    /// # Fields
    /// `metadata` may contain the following optional fields:
    /// - `bookmarks`, a list of strings containing some kind of bookmark identification, e.g
    ///   `["bkmk-transaction:1", "bkmk-transaction:2"]`. Default is `[]`.
    /// - `tx_timeout`, an integer specifying a transaction timeout in milliseconds. Default is the
    ///   server-side configured timeout.
    /// - `tx_metadata`, a map containing some metadata information, mainly used for logging.
    /// - `mode`, a string which specifies what kind of server should be used for this
    ///   transaction. For write access, use `"w"` and for read access use `"r"`. Default is `"w"`.
    /// - `db`, a string containing the name of the database where the transaction should take
    ///   place. [`null`](bolt_proto::Value::Null) and `""` denote the server-side configured
    ///   default database. Default is `null`. _(Bolt v4+ only.)_
    ///
    /// # Response
    /// - [`Message::Success`] - the transaction has been successfully started and the server has
    ///   entered the [`TxReady`](bolt_proto::ServerState::Ready) state.
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server may attach
    ///   metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn begin(&mut self, metadata: Option<Metadata>) -> CommunicationResult<Message> {
        let begin_msg = Begin::new(metadata.unwrap_or_default().value);
        self.send_message(Message::Begin(begin_msg)).await?;
        self.read_message().await
    }

    /// Send a [`COMMIT`](Message::Commit) message to the server.
    /// _(Bolt v3+ only.)_
    ///
    /// # Description
    /// The `COMMIT` message requests to commit the results of an explicit transaction and
    /// transition the server back to the [`Ready`](bolt_proto::ServerState::Ready) state.
    ///
    /// The server must be in the [`TxReady`](bolt_proto::ServerState::TxReady) state to be able to
    /// successfully process a `COMMIT` request, which means that any outstanding results in the
    /// result stream must be consumed via [`Client::pull`]. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `COMMIT` request will be
    /// considered a protocol violation and will lead to connection closure.
    ///
    /// To instead cancel pending changes, send a [`ROLLBACK`](Message::Rollback) message.
    ///
    /// # Response
    /// - [`Message::Success`] - the transaction has been successfully committed and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server sends the
    ///   following metadata fields in the response:
    ///   - `bookmark` (e.g. `"bookmark:1234"`)
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server may attach
    ///   metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn commit(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::Commit).await?;
        self.read_message().await
    }

    /// Send a [`ROLLBACK`](Message::Rollback) message to the server.
    /// _(Bolt v3+ only.)_
    ///
    /// # Description
    /// The `ROLLBACK` message requests to cancel a transaction and transition the server back to
    /// the [`Ready`](bolt_proto::ServerState::Ready) state. Any changes made since the transaction
    /// was started will be undone.
    ///
    /// The server must be in the [`TxReady`](bolt_proto::ServerState::TxReady) state to be able to
    /// successfully process a `ROLLBACK` request, which means that any outstanding results in the
    /// result stream must be consumed via [`Client::pull`]. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `ROLLBACK` request will
    /// be considered a protocol violation and will lead to connection closure.
    ///
    /// To instead persist pending changes, send a [`COMMIT`](Message::Commit) message.
    ///
    /// # Response
    /// - [`Message::Success`] - the transaction has been successfully reverted and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state.
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server may attach
    ///   metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn rollback(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::Rollback).await?;
        self.read_message().await
    }

    /// Send a [`ROUTE`](Message::Route) message to the server.
    /// _(Bolt v4.3+ only.)_
    ///
    /// # Description
    /// The `ROUTE` message instructs the server to return the current routing table.
    ///
    /// The server must be in the [`Ready`](bolt_proto::ServerState::Ready) state to be able to
    /// successfully process a `ROUTE` request. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `ROUTE` request will be
    /// considered a protocol violation and will lead to connection closure.
    ///
    /// # Fields
    /// - `context`, which should contain routing context information as well as an `address`
    ///   field indicating to which address the client should initially connect.
    /// - `bookmarks`, a list of strings containing some kind of bookmark identification, e.g
    ///   `["bkmk-transaction:1", "bkmk-transaction:2"]`. Default is `[]`.
    /// - `database`, a string containing the name of the database for which this command should be
    ///   run. `""` denotes the server-side configured default database.
    ///
    /// # Response
    /// - [`Message::Success`] - the routing table has been successfully retrieved and the server
    ///   has entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server sends the
    ///   following metadata fields in the response:
    ///   - `rt`, a map with the following fields:
    ///     - `ttl`, an integer denoting the number of seconds this routing table should be
    ///       considered valid
    ///     - `servers`, a list of maps representing roles for one or more addresses. Each element
    ///       will have the following fields:
    ///       - `role`, a server role. Possible values are `"READ"`, `"WRITE"`, and `"ROUTE"`.
    ///       - `addresses`, a list of strings representing the servers with the specified role
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server may attach
    ///   metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(4.3)]
    pub async fn route(
        &mut self,
        context: RoutingContext,
        bookmarks: impl Into<Vec<String>>,
        database: impl Into<String>,
    ) -> CommunicationResult<Message> {
        let route_msg = Route::new(context.value, bookmarks.into(), database.into());
        self.send_message(Message::Route(route_msg)).await?;
        self.read_message().await
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

        while !self.sent_queue.is_empty() {
            responses.push(self.read_message().await?);
        }
        Ok(responses)
    }
}
