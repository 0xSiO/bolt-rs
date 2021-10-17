use bolt_client_macros::*;
use bolt_proto::{message::*, Message, ServerState::*};
use futures_util::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{error::CommunicationResult, Client, Metadata, Params};

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    /// Send a [`HELLO`](Message::Hello) message to the server.
    /// _(Bolt v3+ only. For Bolt v1 - v2, see [`Client::init`])._
    ///
    /// # Description
    /// The `HELLO` message requests the connection to be authorized for use with the remote
    /// database.
    ///
    /// The server must be in the [`Connected`](bolt_proto::ServerState::Connected) state to be
    /// able to process a `HELLO` message. For any other states, receipt of a `HELLO` message is
    /// considered a protocol violation and leads to connection closure.
    ///
    /// Clients should send a `HELLO` message to the server immediately after connection and
    /// process the response before using that connection in any other way.
    ///
    /// If authentication fails, the server will respond with a [`FAILURE`](Message::Failure)
    /// message and immediately close the connection. Clients wishing to retry initialization
    /// should establish a new connection.
    ///
    /// # Fields
    /// - `metadata` should contain at least two entries:
    ///   - `user_agent`, a [`String`](bolt_proto::Value::String) which should conform to the
    ///     format `"Name/Version"`, for example `"Example/1.0.0"` (see
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
    ///   - `connection_id` (e.g. `"bolt-61"`)
    /// - [`Message::Failure`] - initialization has failed and the server has entered the
    ///   [`Defunct`](bolt_proto::ServerState::Defunct) state. The server may choose to include
    ///   metadata describing the nature of the failure but will immediately close the connection
    ///   after the failure has been sent.
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn hello(&mut self, metadata: Option<Metadata>) -> CommunicationResult<Message> {
        let hello_msg = Hello::new(metadata.unwrap_or_default().value);
        self.send_message(Message::Hello(hello_msg)).await?;
        self.read_message().await
    }

    /// Send a `GOODBYE` message to the server.
    ///
    /// # Description
    /// The `GOODBYE` message is a Bolt v3+ client message used to end the session. The
    /// server will end the connection upon receipt of this message.
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn goodbye(&mut self) -> CommunicationResult<()> {
        self.send_message(Message::Goodbye).await?;
        self.server_state = Defunct;
        Ok(self.stream.close().await?)
    }

    /// Send a `RUN_WITH_METADATA` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `RUN` for Bolt v3+ clients, but allows passing
    /// an arbitrary metadata hash along with the request.
    ///
    /// # Response
    /// - `SUCCESS {…​}` if the statement has been accepted for execution
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed or
    ///   if a statement may not be executed at this time
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn run_with_metadata(
        &mut self,
        statement: impl Into<String>,
        parameters: Option<Params>,
        metadata: Option<Metadata>,
    ) -> CommunicationResult<Message> {
        let run_msg = RunWithMetadata::new(
            statement.into(),
            parameters.unwrap_or_default().value,
            metadata.unwrap_or_default().value,
        );
        self.send_message(Message::RunWithMetadata(run_msg)).await?;
        self.read_message().await
    }

    /// Send a `BEGIN` message to the server.
    ///
    /// # Description
    /// This Bolt v3+ message begins a transaction. A hash of arbitrary metadata can be
    /// passed along with the request.
    ///
    /// # Response
    /// - `SUCCESS {…}` if transaction has started successfully
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed, or
    ///   if transaction could not be started
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn begin(&mut self, metadata: Option<Metadata>) -> CommunicationResult<Message> {
        let begin_msg = Begin::new(metadata.unwrap_or_default().value);
        self.send_message(Message::Begin(begin_msg)).await?;
        self.read_message().await
    }

    /// Send a `COMMIT` message to the server.
    ///
    /// # Description
    /// This Bolt v3+ message commits a transaction. Any changes made since the
    /// transaction was started will be persisted to the database. To instead cancel
    /// pending changes, send a `ROLLBACK` message.
    ///
    /// # Response
    /// - `SUCCESS {…}` if transaction has been committed successfully
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed, or
    ///   if transaction could not be committed
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn commit(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::Commit).await?;
        self.read_message().await
    }

    /// Send a `ROLLBACK` message to the server.
    ///
    /// # Description
    /// This Bolt v3+ message cancels a transaction. Any changes made since the
    /// transaction was started will be undone. To instead keep pending changes, send a
    /// `COMMIT` message.
    ///
    /// # Response
    /// - `SUCCESS {…}` if transaction has been rolled back successfully
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed, or
    ///   if transaction could not be rolled back
    #[bolt_version(3, 4, 4.1, 4.2, 4.3)]
    pub async fn rollback(&mut self) -> CommunicationResult<Message> {
        self.send_message(Message::Rollback).await?;
        self.read_message().await
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use bolt_proto::{value::*, version::*};

    use crate::{client::v1::tests::*, error::CommunicationError, skip_if_handshake_failed};

    use super::*;

    #[tokio::test]
    async fn hello() {
        let client = new_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Connected);
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn hello_fail() {
        let client = new_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Connected);
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert_eq!(client.server_state(), Defunct);
    }

    #[tokio::test]
    async fn goodbye() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        assert!(client.goodbye().await.is_ok());
        assert_eq!(client.server_state(), Defunct);
    }

    #[tokio::test]
    async fn run_with_metadata() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn run_with_metadata_pipelined() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v3-pipelined'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v3-pipelined'});".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v3-pipelined'}) CREATE (:Library {name: 'bolt-client', v1_release: date('2019-12-23'), test: 'v3-pipelined'})-[:CLIENT_FOR]->(neo4j);".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v3-pipelined'}), (bolt_client:Library {name: 'bolt-client', test: 'v3-pipelined'}) RETURN duration.between(neo4j.v1_release, bolt_client.v1_release);".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
        ];
        for response in client.pipeline(messages).await.unwrap() {
            assert!(match response {
                Message::Success(_) => true,
                Message::Record(record) => {
                    assert_eq!(record.fields()[0], Value::from(Duration::new(118, 7, 0, 0)));
                    true
                }
                _ => false,
            });
        }
    }

    #[tokio::test]
    async fn begin() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = client.begin(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), TxReady);
    }

    #[tokio::test]
    async fn commit_empty_transaction() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        client.begin(None).await.unwrap();
        assert_eq!(client.server_state(), TxReady);
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn commit() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        client.begin(None).await.unwrap();
        assert_eq!(client.server_state(), TxReady);

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v3-commit'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v3-commit'});".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
        ];
        client.pipeline(messages).await.unwrap();
        assert_eq!(client.server_state(), TxReady);
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v3-commit'}) RETURN n;".to_string(),
                Default::default(),
                Default::default(),
            )),
            Message::PullAll,
        ];
        let mut node_exists = false;
        for response in client.pipeline(messages).await.unwrap() {
            if let Message::Record(record) = response {
                let node = Node::try_from(record.fields()[0].clone()).unwrap();
                assert_eq!(node.labels(), &["Database"]);
                node_exists = true;
                break;
            }
        }
        assert!(node_exists);
    }

    #[tokio::test]
    async fn commit_with_no_begin_fails() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(matches!(
            client.commit().await,
            Err(CommunicationError::InvalidState { state: Ready, .. })
        ));
    }

    #[tokio::test]
    async fn rollback_empty_transaction() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        client.begin(None).await.unwrap();
        assert_eq!(client.server_state(), TxReady);
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn rollback() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        client.begin(None).await.unwrap();
        assert_eq!(client.server_state(), TxReady);
        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v3-rollback'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v3-rollback'});".to_string(),
                Default::default(), Default::default())),
            Message::PullAll,
        ];
        client.pipeline(messages).await.unwrap();
        assert_eq!(client.server_state(), TxReady);
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v3-rollback'}) RETURN n;".to_string(),
                Default::default(),
                Default::default(),
            )),
            Message::PullAll,
        ];
        for response in client.pipeline(messages).await.unwrap() {
            // There should be no RECORD messages
            assert!(matches!(response, Message::Success(_)));
        }
    }

    #[tokio::test]
    async fn rollback_with_no_begin_fails() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(matches!(
            client.rollback().await,
            Err(CommunicationError::InvalidState { state: Ready, .. })
        ));
    }

    #[tokio::test]
    async fn reset_internals_pipelined() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();

        let mut messages = client
            .pipeline(vec![
                Message::RunWithMetadata(RunWithMetadata::new(
                    String::from("RETURN 1;"),
                    Default::default(),
                    Default::default(),
                )),
                Message::PullAll,
                Message::RunWithMetadata(RunWithMetadata::new(
                    String::from("RETURN 1;"),
                    Default::default(),
                    Default::default(),
                )),
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
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();

        client
            .run_with_metadata("RETURN 1;", None, None)
            .await
            .unwrap();
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
}
