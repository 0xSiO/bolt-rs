#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    use bolt_proto::{message::*, value::*, version::*, Message};

    use crate::client::v1::tests::*;
    use crate::skip_if_handshake_failed;
    use crate::Metadata;

    #[tokio::test]
    async fn hello() {
        let client = new_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn hello_fail() {
        let client = new_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn goodbye() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(client.goodbye().await.is_ok());
    }

    #[tokio::test]
    async fn run_with_metadata() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok())
    }

    #[tokio::test]
    async fn run_with_metadata_pipelined() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4.1-pipelined'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v4.1-pipelined'});".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v4.1-pipelined'}) CREATE (:Library {name: 'bolt-client', v1_release: date('2019-12-23'), test: 'v4.1-pipelined'})-[:CLIENT_FOR]->(neo4j);".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v4.1-pipelined'}), (bolt_client:Library {name: 'bolt-client', test: 'v4.1-pipelined'}) RETURN duration.between(neo4j.v1_release, bolt_client.v1_release);".to_string(),
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
        let client = get_initialized_client(V4_1).await;
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
        let client = get_initialized_client(V4_1).await;
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
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = client.begin(None).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn commit_empty_transaction() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn commit() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4.1-commit'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v4.1-commit'});".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
        ];
        client.pipeline(messages).await.unwrap();
        let response = client.commit().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4.1-commit'}) RETURN n;".to_string(),
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
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = client.commit().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn rollback_empty_transaction() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn rollback() {
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client.begin(None).await.unwrap();
        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4.1-rollback'}) DETACH DELETE n;".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
            Message::RunWithMetadata(RunWithMetadata::new(
                "CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v4.1-rollback'});".to_string(),
                Default::default(), Default::default())),
            Message::Pull(Pull::new(HashMap::from_iter(vec![("n".to_string(), Value::from(1))]))),
        ];
        client.pipeline(messages).await.unwrap();
        let response = client.rollback().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        let messages = vec![
            Message::RunWithMetadata(RunWithMetadata::new(
                "MATCH (n {test: 'v4.1-rollback'}) RETURN n;".to_string(),
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
        let client = get_initialized_client(V4_1).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = client.rollback().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }
}
