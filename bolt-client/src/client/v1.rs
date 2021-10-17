use std::{convert::TryInto, io};

use bolt_client_macros::*;
use bolt_proto::{message::*, version::*, Message};
use futures_util::io::{AsyncRead, AsyncWrite};

use crate::{error::CommunicationResult, Client, Metadata, Params};

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
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
    /// - `metadata` should contain at least two entries:
    ///   - `user_agent`, which should conform to the format `"Name/Version"`, for example
    ///     `"Example/1.0.0"` (see
    ///     [here](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent)).
    ///   - `scheme` is the authentication scheme. Predefined schemes are `"none"`, `"basic"`, or
    ///     `"kerberos"`.
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
    ///   - `server` (e.g. `"Neo4j/4.3.0"`)
    ///   - `connection_id` (e.g. `"bolt-61"`) _(Bolt v3+ only.)_
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
                    .ok_or(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "missing user_agent",
                    ))?
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
    /// # Response
    /// - [`Message::Success`] - the request has been successfully received and the server has
    ///   entered the [`Streaming`](bolt_proto::ServerState::Streaming) state. Clients should not
    ///   consider a `SUCCESS` response to indicate completion of the execution of the query,
    ///   merely acceptance of it. The server may attach metadata to the message to provide header
    ///   detail for the results that follow. The following fields are defined for inclusion in the
    ///   metadata:
    ///   - `fields` (e.g. `["name", "age"]`)
    ///   - `result_available_after` (e.g. `123`)
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

    /// Send a [`DISCARD_ALL`](Message::DiscardAll) message to the server.
    /// _(Bolt v1 - v3 only. For Bolt v4+, see [`Client::discard`].)_
    ///
    /// # Description
    /// The `DISCARD_ALL` message issues a request to discard the outstanding result and return to
    /// the [`Ready`](bolt_proto::ServerState::Ready) state. A receiving server will not abort the
    /// request but continue to process it without streaming any detail messages to the client.
    ///
    /// The server must be in the [`Streaming`](bolt_proto::ServerState::Streaming) state to be
    /// able to successfully process a `DISCARD_ALL` request. If the server is in the
    /// [`Failed`](bolt_proto::ServerState::Failed) state or
    /// [`Interrupted`](bolt_proto::ServerState::Interrupted) state, the response will be
    /// [`IGNORED`](Message::Ignored). For any other states, receipt of a `DISCARD_ALL` request
    /// will be considered a protocol violation and will lead to connection closure.
    ///
    /// # Response
    /// - [`Message::Success`] - the request has been successfully received and the server has
    ///   entered the [`Ready`](bolt_proto::ServerState::Ready) state. The server may attach
    ///   metadata to the message to provide footer detail for the discarded results.
    ///   The following fields are defined for inclusion in the metadata:
    ///   - `bookmark` (e.g. `"bookmark:1234"`)
    ///   - `result_consumed_after` (e.g. `123`)
    /// - [`Message::Ignored`] - the server is in the [`Failed`](bolt_proto::ServerState::Failed)
    ///   or [`Interrupted`](bolt_proto::ServerState::Interrupted) state, and the request was
    ///   discarded without being processed. No server state change has occurred.
    /// - [`Message::Failure`] - the request could not be processed successfully and the server has
    ///   entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server may attach
    ///   metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(1, 2, 3)]
    pub async fn discard_all(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::DiscardAll).await?;
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
    /// - `(_, `[`Message::Success`]`)` - the request has been successfully processed
    ///   and the server has entered the [`Ready`](bolt_proto::ServerState::Ready) state. The
    ///   server may attach metadata to the `SUCCESS` message to provide footer detail for the
    ///   results. The following fields are defined for inclusion in the metadata:
    ///   - `bookmark` (e.g. `"bookmark:1234"`)
    ///   - `result_consumed_after` (e.g. `123`)
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
    /// - [`Message::Success`] - the request has been successfully received and the server has
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
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{convert::TryFrom, env, iter::FromIterator};

    use bolt_proto::{message::*, value::*, version::*, ServerState::*};
    use tokio::io::BufStream;
    use tokio_util::compat::*;

    use crate::{
        error::{CommunicationError, ConnectionResult, Result},
        skip_if_handshake_failed, stream, Metadata,
    };

    use super::*;

    type Stream = Compat<BufStream<stream::Stream>>;

    pub(crate) async fn new_client(version: u32) -> ConnectionResult<Client<Stream>> {
        Client::new(
            BufStream::new(
                stream::Stream::connect(
                    env::var("BOLT_TEST_ADDR").unwrap(),
                    env::var("BOLT_TEST_DOMAIN").ok(),
                )
                .await?,
            )
            .compat(),
            &[version, 0, 0, 0],
        )
        .await
    }

    pub(crate) async fn initialize_client(
        client: &mut Client<Stream>,
        succeed: bool,
    ) -> CommunicationResult<Message> {
        let username = env::var("BOLT_TEST_USERNAME").unwrap();
        let password = if succeed {
            env::var("BOLT_TEST_PASSWORD").unwrap()
        } else {
            String::from("invalid")
        };

        client
            .hello(Metadata::from_iter(vec![
                ("user_agent", "bolt-client/X.Y.Z"),
                ("scheme", "basic"),
                ("principal", &username),
                ("credentials", &password),
            ]))
            .await
    }

    pub(crate) async fn get_initialized_client(version: u32) -> Result<Client<Stream>> {
        let mut client = new_client(version).await?;
        initialize_client(&mut client, true).await?;
        Ok(client)
    }

    pub(crate) async fn run_invalid_query(
        client: &mut Client<Stream>,
    ) -> CommunicationResult<Message> {
        client
            .run(
                "RETURN invalid query oof as n;",
                Some(Params::from_iter(vec![("some_val", 25.5432)])),
                Some(Metadata::from_iter(vec![("some_key", true)])),
            )
            .await
    }

    pub(crate) async fn run_valid_query(
        client: &mut Client<Stream>,
    ) -> CommunicationResult<Message> {
        client
            .run(
                "RETURN $some_val as n;",
                Some(Params::from_iter(vec![("some_val", 25.5432)])),
                Some(Metadata::from_iter(vec![("some_key", true)])),
            )
            .await
    }

    #[tokio::test]
    async fn init() {
        let client = new_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Connected);
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn init_fail() {
        let client = new_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Connected);
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert_eq!(client.server_state(), Defunct);

        // Messages now fail to send since connection was closed
        let response = initialize_client(&mut client, true).await;
        assert!(matches!(
            response,
            Err(CommunicationError::InvalidState { state: Defunct, .. })
        ));
    }

    #[tokio::test]
    async fn ack_failure() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert_eq!(client.server_state(), Failed);
        let response = client.ack_failure().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn ack_failure_after_ignored() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert_eq!(client.server_state(), Failed);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(matches!(response, Message::Ignored));
        assert_eq!(client.server_state(), Failed);
        let response = client.ack_failure().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn run() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn run_pipelined() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let messages = vec![
            Message::Run(Run::new("MATCH (n {test: 'v1-pipelined'}) DETACH DELETE n;".to_string(), Default::default())),
            Message::PullAll,
            Message::Run(Run::new("CREATE (:Database {name: 'neo4j', born: 2007, test: 'v1-pipelined'});".to_string(), Default::default())),
            Message::PullAll,
            Message::Run(Run::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v1-pipelined'}) CREATE (:Library {name: 'bolt-client', born: 2019, test: 'v1-pipelined'})-[:CLIENT_FOR]->(neo4j);".to_string(),
                Default::default())),
            Message::PullAll,
            Message::Run(Run::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v1-pipelined'}), (bolt_client:Library {name: 'bolt-client', test: 'v1-pipelined'}) RETURN bolt_client.born - neo4j.born;".to_string(),
                Default::default())),
            Message::PullAll,
        ];
        for response in client.pipeline(messages).await.unwrap() {
            assert!(match response {
                Message::Success(_) => true,
                Message::Record(record) => {
                    assert_eq!(record.fields()[0], Value::from(12_i8));
                    true
                }
                _ => false,
            });
        }
    }

    #[tokio::test]
    async fn run_and_pull() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = client
            .run("RETURN 3458376 as n;", None, None)
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);

        let (records, response) = client.pull(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields(), &[Value::from(3_458_376)]);
    }

    #[tokio::test]
    async fn node_and_rel_creation() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client
            .run(
                "MATCH (n {test: 'v1-node-rel'}) DETACH DELETE n;",
                None,
                None,
            )
            .await
            .unwrap();
        client.pull(None).await.unwrap();

        client.run("CREATE (:Client {name: 'bolt-client', test: 'v1-node-rel'})-[:WRITTEN_IN]->(:Language {name: 'Rust', test: 'v1-node-rel'});", None, None).await.unwrap();
        client.pull(None).await.unwrap();
        client
            .run(
                "MATCH (c {test: 'v1-node-rel'})-[r:WRITTEN_IN]->(l) RETURN c, r, l;",
                None,
                None,
            )
            .await
            .unwrap();
        let (records, _response) = client.pull(None).await.unwrap();

        let c = Node::try_from(records[0].fields()[0].clone()).unwrap();
        let r = Relationship::try_from(records[0].fields()[1].clone()).unwrap();
        let l = Node::try_from(records[0].fields()[2].clone()).unwrap();

        assert_eq!(c.labels(), &[String::from("Client")]);
        assert_eq!(
            c.properties().get("name"),
            Some(&Value::from("bolt-client"))
        );
        assert_eq!(l.labels(), &[String::from("Language")]);
        assert_eq!(l.properties().get("name"), Some(&Value::from("Rust")));
        assert_eq!(r.rel_type(), "WRITTEN_IN");
        assert!(r.properties().is_empty());
        assert_eq!(
            (r.start_node_identity(), r.end_node_identity()),
            (c.node_identity(), l.node_identity())
        );
    }

    #[tokio::test]
    async fn discard_all_fail() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        assert!(matches!(
            client.discard_all().await,
            Err(CommunicationError::InvalidState { state: Ready, .. })
        ));
    }

    #[tokio::test]
    async fn discard_all() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
        let response = client.discard_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn discard_all_and_pull() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
        let response = client.discard_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        assert!(matches!(
            client.pull(None).await,
            Err(CommunicationError::InvalidState { state: Ready, .. })
        ));
    }

    #[tokio::test]
    async fn reset() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert_eq!(client.server_state(), Failed);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(matches!(response, Message::Ignored));
        assert_eq!(client.server_state(), Failed);
        let response = client.reset().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn reset_internals_pipelined() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();

        let mut messages = client
            .pipeline(vec![
                Message::Run(Run::new(String::from("RETURN 1;"), Default::default())),
                Message::PullAll,
                Message::Run(Run::new(String::from("RETURN 1;"), Default::default())),
                Message::PullAll,
                Message::Reset,
            ])
            .await
            .unwrap();

        // Last message should be a SUCCESS...
        assert_eq!(
            messages.pop(),
            Some(Message::Success(Success::new(Default::default())))
        );

        // ... preceded by 4 or more IGNORED
        assert!(messages.len() >= 4);
        for message in messages {
            assert_eq!(message, Message::Ignored);
        }
    }

    #[tokio::test]
    async fn reset_internals() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();

        client.run("RETURN 1;", None, None).await.unwrap();
        client.send_message(Message::PullAll).await.unwrap();
        client.send_message(Message::Reset).await.unwrap();
        assert_eq!(client.server_state(), Interrupted);

        // Two situations can happen here - either the PULL_ALL is ignored, or the records of the
        // PULL_ALL are ignored. The latter situation results in additional IGNORED messages in
        // the result stream.

        // RECORD or PULL_ALL summary, it's not consistent
        assert_eq!(client.read_message().await.unwrap(), Message::Ignored);

        match client.read_message().await.unwrap() {
            // PULL_ALL summary
            Message::Ignored => {
                // RESET result
                Success::try_from(client.read_message().await.unwrap()).unwrap();
            }
            // RESET result
            Message::Success(_) => {}
            other => panic!("unexpected response {:?}", other),
        }
    }

    #[tokio::test]
    async fn ignored() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert_eq!(client.server_state(), Failed);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(matches!(response, Message::Ignored));
        assert_eq!(client.server_state(), Failed);
    }

    #[tokio::test]
    async fn v3_method_with_v1_client_fails() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(matches!(
            client.commit().await,
            Err(CommunicationError::UnsupportedOperation(V1_0))
        ));
    }

    #[tokio::test]
    async fn v3_message_with_v1_client_fails() {
        let client = get_initialized_client(V1_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let begin = Begin::new(Default::default());
        client.send_message(Message::Begin(begin)).await.unwrap();
        assert!(matches!(
            client.read_message().await,
            Err(CommunicationError::ProtocolError(
                bolt_proto::error::Error::DeserializationError(
                    bolt_proto::error::DeserializationError::IoError(_)
                )
            ))
        ));
    }
}
