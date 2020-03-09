use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use bytes::*;
use tokio::io::BufStream;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::prelude::*;

use crate::message::*;
use crate::stream::Stream;
use crate::{Message, Value};

pub(crate) type Result<T> = failure::Fallible<T>;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];
const SUPPORTED_VERSIONS: [u32; 4] = [1, 0, 0, 0];

#[derive(Debug)]
pub struct Client {
    pub(crate) stream: BufStream<Stream>,
    pub(crate) version: u8,
}

impl Client {
    /// Create a new client pointing to the provided server address. If a server domain is provided, the Client will
    /// attempt to connect to the server over a connection secured with TLS.
    pub async fn new(addr: impl ToSocketAddrs, domain: Option<&str>) -> Result<Self> {
        let stream = match domain {
            Some(domain) => Stream::SecureTcp(
                async_native_tls::connect(domain, TcpStream::connect(addr).await?).await?,
            ),
            None => Stream::Tcp(TcpStream::connect(addr).await?),
        };
        let mut client = Client {
            stream: BufStream::new(stream),
            version: 0,
        };
        client.version = client.handshake().await? as u8;
        Ok(client)
    }

    async fn handshake(&mut self) -> Result<u32> {
        let mut allowed_versions = BytesMut::with_capacity(16);
        SUPPORTED_VERSIONS
            .iter()
            .for_each(|&v| allowed_versions.put_u32(v));
        self.stream.write(&PREAMBLE).await?;
        self.stream.write_buf(&mut allowed_versions).await?;
        self.stream.flush().await?;
        Ok(self.stream.read_u32().await?)
    }

    pub(crate) async fn read_message(&mut self) -> Result<Message> {
        let message = Message::from_stream(&mut self.stream).await?;

        #[cfg(test)]
        println!("<<< {:?}\n", message);

        Ok(message)
    }

    pub(crate) async fn send_message(&mut self, message: Message) -> Result<()> {
        #[cfg(test)]
        println!(">>> {:?}", message);

        let chunks: Vec<Bytes> = message.try_into()?;
        for mut chunk in chunks {
            self.stream.write_buf(&mut chunk).await?;
        }
        self.stream.flush().await?;
        Ok(())
    }

    // Documentation for message-related instance methods below is copied from the descriptions given by
    // Neo Technology, Inc. on https://boltprotocol.org/v1/, with minor modifications.
    //
    // The below documentation comments are licensed under the Creative Commons Attribution-ShareAlike 3.0 Unported
    // License. To view a copy of this license, visit http://creativecommons.org/licenses/by-sa/3.0/ or send a letter to
    // Creative Commons, PO Box 1866, Mountain View, CA 94042, USA.

    /// Send an `INIT` message to the server.
    ///
    /// # Description
    /// The `INIT` message is a client message used once to initialize the session. This message is always the first
    /// message the client sends after negotiating protocol version via the initial handshake. Sending any message
    /// other than `INIT` as the first message to the server will result in a `FAILURE`. The client must acknowledge
    /// failures using `ACK_FAILURE`, after which `INIT` may be reattempted.
    ///
    /// # Response
    /// - `SUCCESS {}` if initialization has completed successfully
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed, or if initialization
    ///     cannot be performed at this time, or if the authorization failed.
    pub async fn init(
        &mut self,
        client_name: String,
        auth_token: HashMap<String, String>,
    ) -> Result<Message> {
        let init_msg = Init::new(client_name, auth_token);
        self.send_message(Message::from(init_msg)).await?;
        self.read_message().await
    }

    /// Send a `RUN` message to the server.
    ///
    /// # Description
    /// The `RUN` message is a client message used to pass a statement for execution on the server.
    /// On receipt of a `RUN` message, the server will start a new job by executing the statement with the parameters
    /// (optionally) supplied. If successful, the subsequent response will consist of a single `SUCCESS` message; if
    /// not, a `FAILURE` response will be sent instead. A successful job will always produce a result stream which must
    /// then be explicitly consumed (via `PULL_ALL` or `DISCARD_ALL`), even if empty.
    ///
    /// Depending on the statement you are executing, additional metadata may be returned in both the `SUCCESS` message
    /// from the `RUN`, as well as in the final `SUCCESS` after the stream has been consumed. It is up to the statement
    /// you are running to determine what meta data to return. Notably, most queries will contain a `fields` metadata
    /// section in the `SUCCESS` message for the RUN statement, which lists the result record field names, and a
    /// `result_available_after` section measuring the number of milliseconds it took for the results to be available
    /// for consumption.
    ///
    /// In the case where a previous result stream has not yet been fully consumed, an attempt to `RUN` a new job will
    /// trigger a `FAILURE` response.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will immediately respond with a
    /// single `IGNORED` message and take no further action.
    ///
    /// # Response
    /// - `SUCCESS {"fields": …​, "result_available_after"}` if the statement has been accepted for execution
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed or if a statement may not be executed at this
    ///     time
    pub async fn run(
        &mut self,
        statement: String,
        parameters: Option<HashMap<String, Value>>,
    ) -> Result<Message> {
        let run_msg = Run::new(statement, parameters.unwrap_or_default());
        self.send_message(Message::from(run_msg)).await?;
        self.read_message().await
    }

    /// Send multiple messages to the server without waiting for a response. Returns a Vec containing the server's
    /// response messages for each of the sent messages, in the order they were provided.
    ///
    /// # Description
    /// The client is not required to wait for a response before sending more messages. Sending multiple messages
    /// together like this is called pipelining. For performance reasons, it is recommended that clients use pipelining
    /// as much as possible. Through pipelining, multiple messages can be transmitted together in the same network
    /// package, significantly reducing latency and increasing throughput.
    ///
    /// A common technique is to buffer outgoing messages on the client until the last possible moment, such as when a
    /// commit is issued or a result is read by the application, and then sending all messages in the buffer together.
    ///
    /// # Failure Handling
    /// Because the protocol leverages pipelining, the client and the server need to agree on what happens when a
    /// failure occurs, otherwise messages that were sent assuming no failure would occur might have unintended effects.
    ///
    /// When requests fail on the server, the server will send the client a `FAILURE` message. The client must
    /// acknowledge the `FAILURE` message by sending an `ACK_FAILURE` message to the server. Until the server receives
    /// the `ACK_FAILURE` message, it will send an `IGNORED` message in response to any other message from the client,
    /// including messages that were sent in a pipeline.
    pub async fn run_pipelined(&mut self, messages: Vec<Message>) -> Result<Vec<Message>> {
        // This Vec is too small if we're expecting some RECORD messages, so there's no "good" size
        let mut responses = Vec::with_capacity(messages.len());

        for message in messages {
            self.send_message(message).await?;
        }

        for _ in 0..responses.capacity() {
            let mut response = self.read_message().await?;
            while let &Message::Record(_) = &response {
                responses.push(response);
                response = self.read_message().await?;
            }
            responses.push(response);
        }
        Ok(responses)
    }

    /// Send a `DISCARD_ALL` message to the server.
    ///
    /// # Description
    /// The `DISCARD_ALL` message is a client message used to discard all remaining items from the active result stream.
    ///
    /// On receipt of a `DISCARD_ALL` message, the server will dispose of all remaining items from the active result
    /// stream, close the stream and send a single `SUCCESS` message to the client. If no result stream is currently
    /// active, the server will respond with a single `FAILURE` message.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will immediately respond with a
    /// single `IGNORED` message and take no further action.
    ///
    /// # Response
    /// - `SUCCESS {}` if the result stream has been successfully discarded
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently available
    pub async fn discard_all(&mut self) -> Result<Message> {
        self.send_message(Message::DiscardAll).await?;
        self.read_message().await
    }

    /// Send a `PULL_ALL` message to the server. Returns a tuple containing a `Vec` of the records returned from the
    /// server as well as the summary message (`SUCCESS` or `FAILURE`).
    ///
    /// # Description
    /// The `PULL_ALL` message is a client message used to retrieve all remaining items from the active result stream.
    ///
    /// On receipt of a `PULL_ALL` message, the server will send all remaining result data items to the client, each in
    /// a single `RECORD` message. The server will then close the stream and send a single `SUCCESS` message optionally
    /// containing summary information on the data items sent. If an error is encountered, the server must instead send
    /// a `FAILURE` message, discard all remaining data items and close the stream.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will immediately respond with a
    /// single `IGNORED` message and take no further action.
    ///
    /// # Response
    /// - `SUCCESS {…​}` if the result stream has been successfully transferred
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently available or if retrieval fails
    pub async fn pull_all(&mut self) -> Result<(Message, Vec<Record>)> {
        self.send_message(Message::PullAll).await?;
        let mut records = vec![];
        loop {
            match self.read_message().await? {
                Message::Record(record) => records.push(Record::try_from(record)?),
                other => return Ok((other, records)),
            }
        }
    }

    /// Send an `ACK_FAILURE` message to the server.
    ///
    /// # Description
    /// The `ACK_FAILURE` message is a client message used to acknowledge a failure the server has sent.
    ///
    /// The following actions are performed by `ACK_FAILURE`:
    /// - clear any outstanding `FAILURE` state
    ///
    /// In some cases, it may be preferable to use `RESET` after a failure, to clear the entire state of the connection.
    ///
    /// # Response
    /// - `SUCCESS {}` if the session was successfully reset
    /// - `FAILURE {"code": …​, "message": …​}` if there is no failure waiting to be cleared
    pub async fn ack_failure(&mut self) -> Result<Message> {
        self.send_message(Message::AckFailure).await?;
        self.read_message().await
    }

    /// Send a `RESET` message to the server.
    ///
    /// # Description
    /// The `RESET` message is a client message used to return the current session to a "clean" state. It will cause the
    /// session to `IGNORE` any message it is currently processing, as well as any message before `RESET` that had not
    /// yet begun processing. This allows `RESET` to abort long-running operations. It also means clients must be
    /// careful about pipelining `RESET`. Only send this if you are not currently waiting for a result from a prior
    /// message, or if you want to explicitly abort any prior message.
    ///
    /// The following actions are performed by `RESET`:
    /// - force any currently processing message to abort with `IGNORED`
    /// - force any pending messages that have not yet started processing to be `IGNORED`
    /// - clear any outstanding `FAILURE` state
    /// - dispose of any outstanding result records
    /// - rollback the current transaction (if any)
    ///
    /// See [`ack_failure`](Client::ack_failure) for sending a message that only clears `FAILURE` state.
    ///
    /// # Response
    /// - `SUCCESS {}` if the session was successfully reset
    /// - `FAILURE {"code": …​, "message": …​}` if a reset is not currently possible
    pub async fn reset(&mut self) -> Result<Message> {
        self.send_message(Message::Reset).await?;
        self.read_message().await
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::env;
    use std::iter::FromIterator;

    use crate::value::*;

    use super::*;

    async fn new_client() -> Result<Client> {
        let client = Client::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok().as_deref(),
        )
        .await?;
        assert_eq!(client.version, 1);
        Ok(client)
    }

    async fn initialize_client(client: &mut Client, succeed: bool) -> Result<Message> {
        let username = env::var("BOLT_TEST_USERNAME").unwrap();
        let password = if succeed {
            env::var("BOLT_TEST_PASSWORD").unwrap()
        } else {
            "invalid".to_string()
        };

        client
            .init(
                "bolt-client/X.Y.Z".to_string(),
                HashMap::from_iter(vec![
                    (String::from("scheme"), String::from("basic")),
                    (String::from("principal"), username),
                    (String::from("credentials"), password),
                ]),
            )
            .await
    }

    async fn get_initialized_client() -> Result<Client> {
        let mut client = new_client().await?;
        initialize_client(&mut client, true).await?;
        Ok(client)
    }

    async fn run_invalid_query(client: &mut Client) -> Result<Message> {
        client.run("".to_string(), None).await
    }

    async fn run_valid_query(client: &mut Client) -> Result<Message> {
        client.run("RETURN 1 as n;".to_string(), None).await
    }

    #[tokio::test]
    async fn init() {
        let mut client = new_client().await.unwrap();
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn init_fail() {
        let mut client = new_client().await.unwrap();
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ack_failure() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        let response = client.ack_failure().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ack_failure_after_ignored() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(match response {
            Message::Ignored => true,
            _ => false,
        });
        let response = client.ack_failure().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn run() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn run_pipelined() {
        let mut client = get_initialized_client().await.unwrap();
        let messages = vec![
            Message::from(Run::new("MATCH (n) DETACH DELETE n;".to_string(), Default::default())),
            Message::PullAll,
            Message::from(Run::new("CREATE (:Database {name: 'neo4j', born: 2007});".to_string(), Default::default())),
            Message::PullAll,
            Message::from(Run::new(
                "MATCH (neo4j:Database {name: 'neo4j'}) CREATE (:Library {name: 'bolt-client', born: 2019})-[:CLIENT_FOR]->(neo4j);".to_string(),
                Default::default())),
            Message::PullAll,
            Message::from(Run::new(
                "MATCH (neo4j:Database {name: 'neo4j'}), (bolt_client:Library {name: 'bolt-client'}) RETURN bolt_client.born - neo4j.born;".to_string(),
                Default::default())),
            Message::PullAll,
        ];
        for response in client.run_pipelined(messages).await.unwrap() {
            assert!(match response {
                Message::Success(_) => true,
                Message::Record(record) => {
                    assert_eq!(
                        Record::try_from(record).unwrap().fields()[0],
                        Value::from(12_i8)
                    );
                    true
                }
                _ => false,
            });
        }
    }

    #[tokio::test]
    async fn run_and_pull() {
        let mut client = get_initialized_client().await.unwrap();
        let response = client
            .run("RETURN 3458376 as n;".to_string(), None)
            .await
            .unwrap();
        assert!(Success::try_from(response).is_ok());

        let (response, records) = client.pull_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        assert!(!records.is_empty());
        assert_eq!(records[0].fields(), &[Value::from(3458376)]);
    }

    #[tokio::test]
    async fn node_and_rel_creation() {
        let mut client = get_initialized_client().await.unwrap();
        let statement = "MATCH (n) DETACH DELETE n;".to_string();
        client.run(statement, None).await.unwrap();
        client.pull_all().await.unwrap();

        let statement =
            "CREATE (:Client {name: 'bolt-client'})-[:WRITTEN_IN]->(:Language {name: 'Rust'});"
                .to_string();
        client.run(statement, None).await.unwrap();
        client.pull_all().await.unwrap();
        let statement = "MATCH (c)-[r:WRITTEN_IN]->(l) RETURN c, r, l;".to_string();
        client.run(statement, None).await.unwrap();
        let (_response, records) = client.pull_all().await.unwrap();

        let c = Node::try_from(records[0].fields()[0].clone()).unwrap();
        let r = Relationship::try_from(records[0].fields()[1].clone()).unwrap();
        let l = Node::try_from(records[0].fields()[2].clone()).unwrap();

        assert_eq!(c.labels(), &["Client".to_string()]);
        assert_eq!(
            c.properties().get("name"),
            Some(&Value::from("bolt-client"))
        );
        assert_eq!(l.labels(), &["Language".to_string()]);
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
        let mut client = get_initialized_client().await.unwrap();
        let response = client.discard_all().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn discard_all() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = client.discard_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn discard_all_and_pull() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = client.discard_all().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let (response, records) = client.pull_all().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn reset() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(match response {
            Message::Ignored => true,
            _ => false,
        });
        let response = client.reset().await.unwrap();
        assert!(Success::try_from(response).is_ok());
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ignored() {
        let mut client = get_initialized_client().await.unwrap();
        let response = run_invalid_query(&mut client).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
        let response = run_valid_query(&mut client).await.unwrap();
        assert!(match response {
            Message::Ignored => true,
            _ => false,
        });
    }
}
