use bolt_client_macros::*;
use bolt_proto::{message::*, Message, ServerState::*};
use futures_util::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{error::CommunicationResult, Client, Metadata};

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
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
    /// - [`Message::Success`] - the request has been successfully received and the server has
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
    /// - [`Message::Success`] - the request has been successfully received and the server has
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
    async fn run() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn run_pipelined() {
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
}
