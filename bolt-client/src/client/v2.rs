#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

    use bolt_proto::{message::*, value::*, version::*, ServerState::*};

    use crate::{client::v1::tests::*, error::CommunicationError, skip_if_handshake_failed};

    #[tokio::test]
    async fn init() {
        let client = new_client(V2_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Connected);
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
    }

    #[tokio::test]
    async fn init_fail() {
        let client = new_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);
    }

    #[tokio::test]
    async fn run_pipelined() {
        let client = get_initialized_client(V2_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let messages = vec![
            Message::Run(Run::new("MATCH (n {test: 'v2-pipelined'}) DETACH DELETE n;".to_string(), Default::default())),
            Message::PullAll,
            Message::Run(Run::new("CREATE (:Database {name: 'neo4j', v1_release: date('2010-02-16'), test: 'v2-pipelined'});".to_string(), Default::default())),
            Message::PullAll,
            Message::Run(Run::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v2-pipelined'}) CREATE (:Library {name: 'bolt-client', v1_release: date('2019-12-23'), test: 'v2-pipelined'})-[:CLIENT_FOR]->(neo4j);".to_string(),
                Default::default())),
            Message::PullAll,
            Message::Run(Run::new(
                "MATCH (neo4j:Database {name: 'neo4j', test: 'v2-pipelined'}), (bolt_client:Library {name: 'bolt-client', test: 'v2-pipelined'}) RETURN duration.between(neo4j.v1_release, bolt_client.v1_release);".to_string(),
                Default::default())),
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
        let client = get_initialized_client(V2_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert_eq!(client.server_state(), Ready);
        let response = client
            .run(
                "RETURN localdatetime('2010-03-05T12:30:01.000000500');",
                None,
            )
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);

        let (response, records) = client.pull_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].fields(),
            &[Value::from(NaiveDateTime::new(
                NaiveDate::from_ymd(2010, 3, 5),
                NaiveTime::from_hms_nano(12, 30, 1, 500),
            ))]
        );

        let response = client
            .run("RETURN point({x: 42.5123, y: 1.123, z: 3214});", None)
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Streaming);

        let (response, records) = client.pull_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(client.server_state(), Ready);
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].fields(),
            &[Value::from(Point3D::new(9157, 42.5123, 1.123, 3214.0))]
        );
    }

    #[tokio::test]
    async fn node_and_rel_creation() {
        let client = get_initialized_client(V2_0).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        client
            .run("MATCH (n {test: 'v2-node-rel'}) DETACH DELETE n;", None)
            .await
            .unwrap();
        client.pull_all().await.unwrap();

        client.run("CREATE (:Client {name: 'bolt-client', starting: datetime('2019-12-19T16:08:04.322-08:00'), test: 'v2-node-rel'})-[:WRITTEN_IN]->(:Language {name: 'Rust', test: 'v2-node-rel'});", None).await.unwrap();
        client.pull_all().await.unwrap();
        client
            .run(
                "MATCH (c {test: 'v2-node-rel'})-[r:WRITTEN_IN]->(l) RETURN c, r, l;",
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
        assert_eq!(
            c.properties().get("starting"),
            Some(&Value::from(
                FixedOffset::east(-8 * 3600).from_utc_datetime(
                    &NaiveDate::from_ymd(2019, 12, 19).and_hms_milli(16, 8, 4, 322)
                )
            ))
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
        let client = get_initialized_client(V2_0).await;
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
}
