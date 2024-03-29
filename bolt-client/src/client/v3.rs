#[cfg(test)]
mod tests {
    use bolt_proto::{message::*, value::*, version::*, ServerState::*};

    use crate::{client::v1::tests::*, error::CommunicationError, skip_if_handshake_failed};

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
    async fn run_and_pull() {
        let client = get_initialized_client(V3_0).await;
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
    async fn discard_fail() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        assert!(matches!(
            client.discard(None).await,
            Err(CommunicationError::InvalidState { state: Ready, .. })
        ));
    }

    #[tokio::test]
    async fn discard() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
        let response = client.discard(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn discard_and_pull() {
        let client = get_initialized_client(V3_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
        let response = client.discard(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        assert!(matches!(
            client.pull(None).await,
            Err(CommunicationError::InvalidState { state: Ready, .. })
        ));
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
