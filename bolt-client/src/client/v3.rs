use bolt_client_macros::*;
use bolt_proto::message::*;
use bolt_proto::Message;
use futures_util::io::{AsyncRead, AsyncWrite};

use crate::error::*;
use crate::{Client, Metadata, Params};

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    /// Send a `HELLO` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `INIT` for Bolt v3+ clients, but the client name
    /// and auth token are merged into a single metadata object.
    ///
    /// # Response
    /// - `SUCCESS {…}` if initialization has completed successfully
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed, or
    ///   if initialization cannot be performed at this time, or if the authorization
    ///   failed.
    #[bolt_version(3, 4, 4.1)]
    pub async fn hello(&mut self, metadata: Option<Metadata>) -> Result<Message> {
        let hello_msg = Hello::new(metadata.unwrap_or_default().value);
        self.send_message(Message::Hello(hello_msg)).await?;
        self.read_message().await
    }

    /// Send a `GOODBYE` message to the server.
    ///
    /// # Description
    /// The `GOODBYE` message is a Bolt v3+ client message used to end the session. The
    /// server will end the connection upon receipt of this message.
    #[bolt_version(3, 4, 4.1)]
    pub async fn goodbye(&mut self) -> Result<()> {
        self.send_message(Message::Goodbye).await?;
        Ok(())
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
    #[bolt_version(3, 4, 4.1)]
    pub async fn run_with_metadata(
        &mut self,
        statement: impl Into<String>,
        parameters: Option<Params>,
        metadata: Option<Metadata>,
    ) -> Result<Message> {
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
    #[bolt_version(3, 4, 4.1)]
    pub async fn begin(&mut self, metadata: Option<Metadata>) -> Result<Message> {
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
    #[bolt_version(3, 4, 4.1)]
    pub async fn commit(&mut self) -> Result<Message> {
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
    #[bolt_version(3, 4, 4.1)]
    pub async fn rollback(&mut self) -> Result<Message> {
        self.send_message(Message::Rollback).await?;
        self.read_message().await
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use bolt_proto::{value::*, version::*};

    use crate::client::v1::tests::*;
    use crate::skip_if_handshake_failed;

    use super::*;

    #[tokio::test]
    async fn hello() {
        let client = new_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn hello_fail() {
        let client = new_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn goodbye() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(client.goodbye().await.is_ok());
    }

    #[tokio::test]
    async fn run_with_metadata() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok())
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
        let response = client.begin(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn commit_empty_transaction() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn commit() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();

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
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());

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
        let response = client.commit().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn rollback_empty_transaction() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn rollback() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
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
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());

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
        let response = client.rollback().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }
}
