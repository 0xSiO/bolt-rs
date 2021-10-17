use bolt_client_macros::*;
use bolt_proto::{message::*, Message};
use futures_util::io::{AsyncRead, AsyncWrite};

use crate::{error::CommunicationResult, Client, Metadata, Params};

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    /// Send an [`INIT`](Message::Init) message to the server.
    /// _(Bolt v1 - v2 only. For Bolt v3+, see [`Client::hello`])._
    ///
    /// # Description
    /// The `INIT` message is a request for the connection to be authorized for use with the remote
    /// database.
    ///
    /// The server must be in the [`Connected`](bolt_proto::ServerState::Connected) state to be
    /// able to process an `INIT` request. For any other states, receipt of an `INIT` request is
    /// considered a protocol violation and leads to connection closure.
    ///
    /// Clients should send `INIT` requests to the server immediately after connection and process
    /// the response before using that connection in any other way.
    ///
    /// The `auth_token` is used by the server to determine whether the client is permitted to
    /// exchange further messages. If this authentication fails, the server will respond with a
    /// [`FAILURE`](Message::Failure) message and immediately close the connection. Clients
    /// wishing to retry initialization should establish a new connection.
    ///
    /// # Fields
    ///
    /// - `user_agent` should conform to `"Name/Version"`, for example `"Example/1.0.0"` (see
    ///   <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent>).
    /// - `auth_token` must contain either just the entry `{"scheme" : "none"}` or the keys
    ///   `scheme`, `principal` and `credentials`.
    ///   - `scheme` is the authentication scheme. Predefined schemes are `"none"` or `"basic"`. If
    ///     no `scheme` is provided, it defaults to `"none"`.
    ///
    /// # Response
    /// - [`Message::Success`] - indicates that the client is permitted to exchange further
    ///   messages. Servers may include metadata that describes details of the server environment
    ///   and/or the connection. The following fields are defined for inclusion in the `SUCCESS`
    ///   metadata:
    ///   - `server` (e.g. "Neo4j/3.4.0")
    /// - [`Message::Failure`] - indicates that the client is not permitted to exchange further
    ///   messages. Servers may choose to include metadata describing the nature of the failure but
    ///   will immediately close the connection after the failure has been sent.
    #[bolt_version(1, 2)]
    pub async fn init(
        &mut self,
        user_agent: impl Into<String>,
        auth_token: Metadata,
    ) -> CommunicationResult<Message> {
        let init_msg = Init::new(user_agent.into(), auth_token.value);
        self.send_message(Message::Init(init_msg)).await?;
        self.read_message().await
    }

    /// Send a [`RUN`](Message::Run) message to the server.
    /// _(Bolt v1 - v2 only. For Bolt v3+, see [`Client::run_with_metadata`])._
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
    ///
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
    /// - [`Message::Failure`] - the request cannot be processed successfully or is invalid, and
    ///   the server has entered the [`Failed`](bolt_proto::ServerState::Failed) state. The server
    ///   may attach metadata to the message to provide more detail on the nature of the failure.
    #[bolt_version(1, 2)]
    pub async fn run(
        &mut self,
        query: impl Into<String>,
        parameters: Option<Params>,
    ) -> CommunicationResult<Message> {
        let run_msg = Run::new(query.into(), parameters.unwrap_or_default().value);
        self.send_message(Message::Run(run_msg)).await?;
        self.read_message().await
    }

    /// Send a `DISCARD_ALL` message to the server.
    ///
    /// # Description
    /// The `DISCARD_ALL` message is a Bolt v1 - v3 client message used to discard all
    /// remaining items from the active result stream. For Bolt v4+, see
    /// [`discard`](Client::discard).
    ///
    /// On receipt of a `DISCARD_ALL` message, the server will dispose of all remaining
    /// items from the active result stream, close the stream and send a single `SUCCESS`
    /// message to the client. If no result stream is currently active, the server will
    /// respond with a single `FAILURE` message.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will
    /// immediately respond with a single `IGNORED` message and take no further action.
    ///
    /// # Response
    /// - `SUCCESS {…}` if the result stream has been successfully discarded
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently
    ///   available
    #[bolt_version(1, 2, 3)]
    pub async fn discard_all(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::DiscardAll).await?;
        self.read_message().await
    }

    /// Send a `PULL_ALL` message to the server. Returns a tuple containing a [`Vec`] of
    /// the records returned from the server as well as the summary message (`SUCCESS` or
    /// `FAILURE`).
    ///
    /// # Description
    /// The `PULL_ALL` message is a Bolt v1 - v3 client message used to retrieve all
    /// remaining items from the active result stream. For Bolt v4+, see
    /// [`pull`](Client::pull).
    ///
    /// On receipt of a `PULL_ALL` message, the server will send all remaining result data
    /// items to the client, each in a single `RECORD` message. The server will then close
    /// the stream and send a single `SUCCESS` message optionally containing summary
    /// information on the data items sent. If an error is encountered, the server must
    /// instead send a `FAILURE` message, discard all remaining data items and close the
    /// stream.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will
    /// immediately respond with a single `IGNORED` message and take no further action.
    ///
    /// # Response
    /// - `SUCCESS {…​}` if the result stream has been successfully transferred
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently
    ///   available or if retrieval fails
    #[bolt_version(1, 2, 3)]
    pub async fn pull_all(&mut self) -> CommunicationResult<(Message, Vec<Record>)> {
        self.send_message(Message::PullAll).await?;
        let mut records = vec![];
        loop {
            match self.read_message().await? {
                Message::Record(record) => records.push(record),
                Message::Success(success) => return Ok((Message::Success(success), records)),
                Message::Failure(failure) => return Ok((Message::Failure(failure), records)),
                Message::Ignored => return Ok((Message::Ignored, vec![])),
                _ => unreachable!(),
            }
        }
    }

    /// Send an `ACK_FAILURE` message to the server.
    ///
    /// # Description
    /// The `ACK_FAILURE` message is a Bolt v1 - v2 client message used to acknowledge a
    /// failure the server has sent.
    ///
    /// The following actions are performed by `ACK_FAILURE`:
    /// - clear any outstanding `FAILURE` state
    ///
    /// In some cases, it may be preferable to use `RESET` after a failure, to clear the
    /// entire state of the connection.
    ///
    /// # Response
    /// - `SUCCESS {…}` if the session was successfully reset
    /// - `FAILURE {"code": …​, "message": …​}` if there is no failure waiting
    ///   to be cleared
    #[bolt_version(1, 2)]
    pub async fn ack_failure(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::AckFailure).await?;
        self.read_message().await
    }

    /// Send a `RESET` message to the server.
    ///
    /// # Description
    /// The `RESET` message is a client message used to return the current session to a
    /// "clean" state. It will cause the session to `IGNORE` any message it is currently
    /// processing, as well as any message before `RESET` that had not yet begun
    /// processing. This allows `RESET` to abort long-running operations. It also means
    /// clients must be careful about pipelining `RESET`. Only send this if you are not
    /// currently waiting for a result from a prior message, or if you want to explicitly
    /// abort any prior message.
    ///
    /// The following actions are performed by `RESET`:
    /// - force any currently processing message to abort with `IGNORED`
    /// - force any pending messages that have not yet started processing to be `IGNORED`
    /// - clear any outstanding `FAILURE` state
    /// - dispose of any outstanding result records
    /// - rollback the current transaction (if any)
    ///
    /// For Bolt v1 - v2, see [`ack_failure`](Client::ack_failure) for sending a message
    /// that only clears `FAILURE` state.
    ///
    /// # Response
    /// - `SUCCESS {…}` if the session was successfully reset
    /// - `FAILURE {"code": …​, "message": …​}` if a reset is not currently
    ///   possible
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

        let version = client.version();
        if [V1_0, V2_0].contains(&version) {
            client
                .init(
                    "bolt-client/X.Y.Z",
                    Metadata::from_iter(vec![
                        ("scheme", "basic"),
                        ("principal", &username),
                        ("credentials", &password),
                    ]),
                )
                .await
        } else {
            client
                .hello(Some(Metadata::from_iter(vec![
                    ("user_agent", "bolt-client/X.Y.Z"),
                    ("scheme", "basic"),
                    ("principal", &username),
                    ("credentials", &password),
                ])))
                .await
        }
    }

    pub(crate) async fn get_initialized_client(version: u32) -> Result<Client<Stream>> {
        let mut client = new_client(version).await?;
        initialize_client(&mut client, true).await?;
        Ok(client)
    }

    pub(crate) async fn run_invalid_query(
        client: &mut Client<Stream>,
    ) -> CommunicationResult<Message> {
        if client.version() > V2_0 {
            client
                .run_with_metadata(
                    "RETURN invalid query oof as n;",
                    Some(Params::from_iter(vec![("some_val", 25.5432)])),
                    Some(Metadata::from_iter(vec![("some_key", true)])),
                )
                .await
        } else {
            client.run("", None).await
        }
    }

    pub(crate) async fn run_valid_query(
        client: &mut Client<Stream>,
    ) -> CommunicationResult<Message> {
        if client.version() > V2_0 {
            client
                .run_with_metadata(
                    "RETURN $some_val as n;",
                    Some(Params::from_iter(vec![("some_val", 25.5432)])),
                    Some(Metadata::from_iter(vec![("some_key", true)])),
                )
                .await
        } else {
            client.run("RETURN 1 as n;", None).await
        }
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
        let response = client.run("RETURN 3458376 as n;", None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);

        let (response, records) = client.pull_all().await.unwrap();
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
            .run("MATCH (n {test: 'v1-node-rel'}) DETACH DELETE n;", None)
            .await
            .unwrap();
        client.pull_all().await.unwrap();

        client.run("CREATE (:Client {name: 'bolt-client', test: 'v1-node-rel'})-[:WRITTEN_IN]->(:Language {name: 'Rust', test: 'v1-node-rel'});", None).await.unwrap();
        client.pull_all().await.unwrap();
        client
            .run(
                "MATCH (c {test: 'v1-node-rel'})-[r:WRITTEN_IN]->(l) RETURN c, r, l;",
                None,
            )
            .await
            .unwrap();
        let (_response, records) = client.pull_all().await.unwrap();

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
            client.pull_all().await,
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

        client.run("RETURN 1;", None).await.unwrap();
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
