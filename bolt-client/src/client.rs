use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use bytes::*;
use tokio::io::BufStream;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::prelude::*;

use crate::message::*;
use crate::{Message, Value};

pub(crate) type Result<T> = failure::Fallible<T>;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];
const SUPPORTED_VERSIONS: [u32; 4] = [1, 0, 0, 0];

pub struct Client {
    pub(crate) stream: BufStream<TcpStream>,
    pub(crate) version: u8,
}

impl Client {
    /// Create a new connection to the server at the given host and port.
    pub async fn new(addr: impl ToSocketAddrs) -> Result<Self> {
        let mut client = Client {
            stream: BufStream::new(TcpStream::connect(addr).await?),
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
        Message::from_stream(&mut self.stream).await
    }

    pub(crate) async fn send_message(&mut self, message: Message) -> Result<()> {
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
        auth_token: HashMap<String, Value>,
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

    // TODO: Pipelined runs for multiple statements

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
    use std::iter::FromIterator;

    use super::*;

    async fn new_client() -> Result<Client> {
        let client = Client::new("127.0.0.1:7687").await?;
        assert_eq!(client.version, 1);
        Ok(client)
    }

    async fn initialize_client(client: &mut Client, credentials: &str) -> Result<Message> {
        client
            .init(
                "bolt-client/X.Y.Z".to_string(),
                HashMap::from_iter(vec![
                    (String::from("scheme"), Value::from("basic")),
                    (String::from("principal"), Value::from("neo4j")),
                    (String::from("credentials"), Value::from(credentials)),
                ]),
            )
            .await
    }

    async fn get_initialized_client() -> Result<Client> {
        let mut client = new_client().await?;
        initialize_client(&mut client, "test").await?;
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
        let response = initialize_client(&mut client, "test").await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn init_fail() {
        let mut client = new_client().await.unwrap();
        let response = initialize_client(&mut client, "invalid!").await.unwrap();
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

    // TODO: Node/Relationship creation tests

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
