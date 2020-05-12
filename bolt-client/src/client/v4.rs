use std::collections::HashMap;

use bolt_client_macros::*;
use bolt_proto::message::*;
use bolt_proto::{Message, Value};

use crate::error::*;
use crate::Client;

impl Client {
    /// Send a `DISCARD` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `DISCARD_ALL` for Bolt v4+ clients, but allows passing an arbitrary metadata
    /// hash along with the request.
    ///
    /// # Response
    /// - `SUCCESS {…}` if the result stream has been successfully discarded
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently available
    #[bolt_version(4)]
    pub async fn discard(
        &mut self,
        metadata: HashMap<String, impl Into<Value>>,
    ) -> Result<Message> {
        let discard_msg = Discard::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Discard(discard_msg)).await?;
        self.read_message().await
    }

    /// Send a `PULL` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `PULL_ALL` for Bolt v4+ clients, but allows passing an arbitrary metadata hash
    /// along with the request.
    ///
    /// # Response
    /// - `SUCCESS {…​}` if the result stream has been successfully transferred
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently available or if retrieval fails
    #[bolt_version(4)]
    pub async fn pull(
        &mut self,
        metadata: HashMap<String, impl Into<Value>>,
    ) -> Result<(Message, Vec<Record>)> {
        let pull_msg = Pull::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Pull(pull_msg)).await?;
        let mut records = vec![];
        loop {
            match self.read_message().await? {
                Message::Record(record) => records.push(record),
                other => return Ok((other, records)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    use bolt_proto::value::*;

    use crate::client::v1::tests::*;
    use crate::skip_if_handshake_failed;

    use super::*;

    // Current Neo4j behavior:
    //   - Sending DISCARD without 'n' metadata parameter results in a Neo.ClientError.Request.Invalid, saying
    //     "Expecting DISCARD size n to be a Long value, but got: NO_VALUE"
    //   - Sending DISCARD with 'n' equal to some number results in a Neo.DatabaseError.General.UnknownError, saying
    //     "Currently it is only supported to discard ALL records, but it was requested to discard " + n
    //   - Sending DISCARD with 'n' equal to -1 indicates discard of all records in the result stream.
    //
    // This makes it functionally equivalent to DISCARD_ALL... so... why did they do this...?
    #[tokio::test]
    async fn discard() {
        let client = get_initialized_client(4).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = client
            .discard(HashMap::from_iter(vec![("n".to_string(), -1)]))
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    // Current Neo4j behavior:
    //   - Need to send an 'n' metadata parameter here too, but finite values of n will work here.
    #[tokio::test]
    async fn run_and_pull() {
        let client = get_initialized_client(4).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = client
            .run_with_metadata("RETURN 3458376 as n;".to_string(), None, None)
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());

        let (response, records) = client
            .pull(HashMap::from_iter(vec![("n".to_string(), 1)]))
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].fields(), &[Value::from(3_458_376)]);
    }

    #[tokio::test]
    async fn run_with_metadata_pipelined() {
        let client = get_initialized_client(4).await;
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
                    assert_eq!(
                        Record::try_from(record).unwrap().fields()[0],
                        Value::from(Duration::new(118, 7, 0, 0))
                    );
                    true
                }
                _ => false,
            });
        }
    }
}
