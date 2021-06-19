use bolt_client_macros::*;
use bolt_proto::{message::*, Message, ServerState::*, Value};
use futures_util::io::{AsyncRead, AsyncWrite};

use crate::{error::*, require_state, Client, Metadata};

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    /// Send a `DISCARD` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `DISCARD_ALL` for Bolt v4+ clients, but allows
    /// passing an arbitrary metadata hash along with the request.
    ///
    /// # Response
    /// - `SUCCESS {…}` if the result stream has been successfully discarded
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently
    ///   available
    #[bolt_version(4, 4.1)]
    pub async fn discard(&mut self, metadata: Option<Metadata>) -> Result<Message> {
        require_state!(self, Streaming | TxStreaming | Failed | Interrupted);

        let discard_msg = Discard::new(metadata.unwrap_or_default().value);
        self.send_message(Message::Discard(discard_msg)).await?;
        let response = self.read_message().await?;

        match (self.server_state, &response) {
            (Streaming, &Message::Success(ref success)) => {
                // TODO: Build this field into the Success message type
                let has_more = success
                    .metadata()
                    .get("has_more")
                    .cloned()
                    .unwrap_or_else(|| Value::from(false));

                if has_more == Value::from(true) {
                    self.server_state = Streaming;
                } else {
                    self.server_state = Ready;
                }
            }
            (Streaming, &Message::Failure(_)) => self.server_state = Failed,
            (TxStreaming, &Message::Success(ref success)) => {
                let has_more = success
                    .metadata()
                    .get("has_more")
                    .cloned()
                    .unwrap_or_else(|| Value::from(false));

                if has_more == Value::from(true) {
                    self.server_state = TxStreaming;
                } else {
                    // TODO: Or TxStreaming, if there are other streams open??
                    self.server_state = TxReady;
                }
            }
            (TxStreaming, &Message::Failure(_)) => self.server_state = Failed,
            (Failed, &Message::Ignored) => self.server_state = Failed,
            (Interrupted, &Message::Ignored) => self.server_state = Interrupted,
            (state, msg) => {
                self.server_state = Defunct;
                return Err(Error::InvalidResponse(state, msg.clone()));
            }
        }

        Ok(response)
    }

    /// Send a `PULL` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `PULL_ALL` for Bolt v4+ clients, but allows
    /// passing an arbitrary metadata hash along with the request.
    ///
    /// # Response
    /// - `SUCCESS {…​}` if the result stream has been successfully transferred
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently
    ///   available or if retrieval fails
    #[bolt_version(4, 4.1)]
    pub async fn pull(&mut self, metadata: Option<Metadata>) -> Result<(Message, Vec<Record>)> {
        require_state!(self, Streaming | TxStreaming | Failed | Interrupted);

        let pull_msg = Pull::new(metadata.unwrap_or_default().value);
        self.send_message(Message::Pull(pull_msg)).await?;
        let mut records = vec![];
        loop {
            match (self.server_state, self.read_message().await?) {
                (Streaming, Message::Record(record)) => records.push(record),
                (TxStreaming, Message::Record(record)) => records.push(record),
                (Streaming, Message::Success(success)) => {
                    // TODO: Build this field into the Success message type
                    let has_more = success
                        .metadata()
                        .get("has_more")
                        .cloned()
                        .unwrap_or_else(|| Value::from(false));

                    if has_more == Value::from(true) {
                        self.server_state = Streaming;
                    } else {
                        self.server_state = Ready;
                    }

                    return Ok((Message::Success(success), records));
                }
                (Streaming, Message::Failure(failure)) => {
                    self.server_state = Failed;
                    // TODO: Should we return the records, even if they're considered invalid?
                    return Ok((Message::Failure(failure), vec![]));
                }
                (TxStreaming, Message::Success(success)) => {
                    let has_more = success
                        .metadata()
                        .get("has_more")
                        .cloned()
                        .unwrap_or_else(|| Value::from(false));

                    if has_more == Value::from(true) {
                        self.server_state = TxStreaming;
                    } else {
                        // TODO: Or TxStreaming, if there are other streams open??
                        self.server_state = TxReady;
                    }

                    return Ok((Message::Success(success), records));
                }
                (TxStreaming, Message::Failure(failure)) => {
                    self.server_state = Failed;
                    return Ok((Message::Failure(failure), vec![]));
                }
                (Failed, Message::Ignored) => {
                    self.server_state = Failed;
                    return Ok((Message::Ignored, vec![]));
                }
                (Interrupted, Message::Ignored) => {
                    self.server_state = Interrupted;
                    return Ok((Message::Ignored, vec![]));
                }
                (state, msg) => {
                    self.server_state = Defunct;
                    return Err(Error::InvalidResponse(state, msg));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    use bolt_proto::{value::*, version::*};

    use crate::client::v1::tests::*;
    use crate::skip_if_handshake_failed;

    use super::*;

    #[tokio::test]
    async fn hello() {
        let client = new_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn hello_fail() {
        let client = new_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn goodbye() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(client.goodbye().await.is_ok());
    }

    #[tokio::test]
    async fn run_with_metadata() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok())
    }

    #[tokio::test]
    async fn run_with_metadata_pipelined() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4-pipelined'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v4-pipelined'});".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v4-pipelined'}) CREATE (:Library {name: 'bolt-client', v1_release: date('2019-12-23'), test: 'v4-pipelined'})-[:CLIENT_FOR]->(neo4j);".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v4-pipelined'}), (bolt_client:Library {name: 'bolt-client', test: 'v4-pipelined'}) RETURN duration.between(neo4j.v1_release, bolt_client.v1_release);".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
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

    // Current Neo4j behavior:
    //   - Sending DISCARD without 'n' metadata parameter results in a
    //     Neo.ClientError.Request.Invalid, saying "Expecting DISCARD size n to be a Long
    //     value, but got: NO_VALUE"
    //   - Sending DISCARD with 'n' equal to some number results in a
    //     Neo.DatabaseError.General.UnknownError, saying "Currently it is only supported
    //     to discard ALL records, but it was requested to discard " + n
    //   - Sending DISCARD with 'n' equal to -1 indicates discard of all records in the
    //     result stream.
    #[tokio::test]
    async fn discard() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();

        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = client.discard(None).await.unwrap();
        assert!(Failure::try_from(response).is_ok());

        let response = client.reset().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = client
            .discard(Some(Metadata::from_iter(vec![("n", 1)])))
            .await
            .unwrap();
        assert!(Failure::try_from(response).is_ok());

        let response = client.reset().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = client
            .discard(Some(Metadata::from_iter(vec![("n", -1)])))
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    // Current Neo4j behavior:
    //   - Need to send an 'n' metadata parameter here too, but finite values of n will
    //     work here.
    #[tokio::test]
    async fn run_and_pull() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();

        // Try pulling 1 result
        let response = client
            .run_with_metadata("RETURN 3458376 as n;", None, None)
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());

        let (response, records) = client
            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields(), &[Value::from(3_458_376)]);

        // Try pulling all results
        let response = client
            .run_with_metadata("RETURN 3458376 as n;", None, None)
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());

        let (response, records) = client
            .pull(Some(Metadata::from_iter(vec![("n", -1)])))
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields(), &[Value::from(3_458_376)]);
    }

    #[tokio::test]
    async fn begin() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = client.begin(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn commit_empty_transaction() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn commit() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4-commit'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v4-commit'});".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
        ];
        client.pipeline(messages).await.unwrap();
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4-commit'}) RETURN n;".to_string(),
                Default::default(),
                Default::default(),
            )),
            Message::Pull(Pull::new(HashMap::from_iter(vec![(
                "n".to_string(),
                Value::from(1),
            )]))),
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
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(matches!(
            client.commit().await,
            Err(Error::InvalidState(Ready))
        ));
    }

    #[tokio::test]
    async fn rollback_empty_transaction() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn rollback() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4-rollback'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v4-rollback'});".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
        ];
        client.pipeline(messages).await.unwrap();
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4-rollback'}) RETURN n;".to_string(),
                Default::default(),
                Default::default(),
            )),
            Message::Pull(Pull::new(HashMap::from_iter(vec![(
                "n".to_string(),
                Value::from(1),
            )]))),
        ];
        for response in client.pipeline(messages).await.unwrap() {
            // There should be no RECORD messages
            assert!(matches!(response, Message::Success(_)));
        }
    }

    #[tokio::test]
    async fn rollback_with_no_begin_fails() {
        let client = get_initialized_client(V4_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(matches!(
            client.rollback().await,
            Err(Error::InvalidState(Ready))
        ));
    }
}
