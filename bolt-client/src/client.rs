use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::net::IpAddr;

use bytes::*;
use failure::Error;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio::prelude::*;

use bolt_proto::message::*;
use bolt_proto::{Message, Value};
use std::net::Shutdown::Read;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];
const SUPPORTED_VERSIONS: [u32; 4] = [1, 0, 0, 0];

pub struct Client {
    pub(crate) stream: BufStream<TcpStream>,
    pub(crate) version: u8,
}

impl Client {
    pub async fn new(host: IpAddr, port: usize) -> Result<Self, Error> {
        let mut client = Client {
            stream: BufStream::new(TcpStream::connect(format!("{}:{}", host, port)).await?),
            version: 0,
        };
        client.version = client.handshake().await? as u8;
        Ok(client)
    }

    async fn handshake(&mut self) -> Result<u32, Error> {
        let mut allowed_versions = BytesMut::with_capacity(16);
        SUPPORTED_VERSIONS
            .iter()
            .for_each(|&v| allowed_versions.put_u32(v));
        self.stream.write(&PREAMBLE).await?;
        self.stream.write_buf(&mut allowed_versions).await?;
        self.stream.flush().await?;
        Ok(self.stream.read_u32().await?)
    }

    pub async fn read_message(&mut self) -> Result<Message, Error> {
        Message::from_stream(&mut self.stream).await
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), Error> {
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
    /// The `INIT` message is a client message used once to initialize the session. This message is always the first
    /// message the client sends after negotiating protocol version via the initial handshake. Sending any message
    /// other than `INIT` as the first message to the server will result in a `FAILURE`. The client must acknowledge
    /// failures using `ACK_FAILURE`, after which `INIT` may be reattempted.
    ///
    /// Response:
    /// - `SUCCESS {}` if initialization has completed successfully
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed, or if initialization
    ///     cannot be performed at this time, or if the authorization failed.
    pub async fn init(
        &mut self,
        client_name: String,
        auth_token: HashMap<String, Value>,
    ) -> Result<Message, Error> {
        let init_msg = Init::new(client_name, auth_token);
        self.send_message(Message::from(init_msg)).await?;
        Ok(self.read_message().await?)
    }

    /// Send a `RUN` message to the server.
    ///
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
    /// Response:
    /// - `SUCCESS {"fields": …​, "result_available_after"}` if the statement has been accepted for execution
    /// - `FAILURE {"code": …​, "message": …​}` if the request was malformed or if a statement may not be executed at this
    ///     time
    pub async fn run(
        &mut self,
        statement: String,
        parameters: Option<HashMap<String, Value>>,
    ) -> Result<Message, Error> {
        let run_msg = Run::new(statement, parameters.unwrap_or_default());
        self.send_message(Message::from(run_msg)).await?;
        Ok(self.read_message().await?)
    }

    /// Send a `DISCARD_ALL` message to the server.
    ///
    /// The `DISCARD_ALL` message is a client message used to discard all remaining items from the active result stream.
    ///
    /// On receipt of a `DISCARD_ALL` message, the server will dispose of all remaining items from the active result
    /// stream, close the stream and send a single `SUCCESS` message to the client. If no result stream is currently
    /// active, the server will respond with a single `FAILURE` message.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will immediately respond with a
    /// single `IGNORED` message and take no further action.
    ///
    /// Response:
    /// - `SUCCESS {}` if the result stream has been successfully discarded
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently available
    pub async fn discard_all(&mut self) -> Result<Message, Error> {
        self.send_message(Message::DiscardAll).await?;
        Ok(self.read_message().await?)
    }

    /// Send a PULL_ALL message to the server. Returns a tuple containing a Vec of the records returned from the server
    /// as well as the summary message (SUCCESS or FAILURE).
    ///
    /// The PULL_ALL message is a client message used to retrieve all remaining items from the active result stream.
    ///
    /// On receipt of a PULL_ALL message, the server will send all remaining result data items to the client, each in a
    /// single RECORD message. The server will then close the stream and send a single SUCCESS message optionally
    /// containing summary information on the data items sent. If an error is encountered, the server must instead send
    /// a FAILURE message, discard all remaining data items and close the stream.
    ///
    /// If an unacknowledged failure is pending from a previous exchange, the server will immediately respond with a
    /// single IGNORED message and take no further action.
    ///
    /// Response:
    /// - `SUCCESS {…​}` if the result stream has been successfully transferred
    /// - `FAILURE {"code": …​, "message": …​}` if no result stream is currently available or if retrieval fails
    pub async fn pull_all(&mut self) -> Result<(Vec<Record>, Message), Error> {
        self.send_message(Message::PullAll).await?;
        let mut records = vec![];
        loop {
            match self.read_message().await? {
                Message::Record(record) => records.push(Record::try_from(record)?),
                other => return Ok((records, other)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    use bolt_proto::message::*;
    use bolt_proto::Value;

    use super::*;

    async fn new_client() -> Result<Client, Error> {
        Client::new("127.0.0.1".parse().unwrap(), 7687).await
    }

    async fn init_client(credentials: &str) -> Result<Client, Error> {
        let mut client = new_client().await?;
        assert!(client
            .send_message(Message::from(Init::new(
                "bolt-client/X.Y.Z".to_string(),
                HashMap::from_iter(vec![
                    (String::from("scheme"), Value::from("basic")),
                    (String::from("principal"), Value::from("neo4j")),
                    (String::from("credentials"), Value::from(credentials)),
                ]),
            )))
            .await
            .is_ok());
        Ok(client)
    }

    async fn get_initialized_client() -> Result<Client, Error> {
        let mut client = init_client("test").await?;
        let success = client.read_message().await?;
        assert!(Success::try_from(success).is_ok());
        Ok(client)
    }

    async fn send_invalid_message(client: &mut Client) {
        let invalid_msg = Message::from(Run::new("".to_string(), HashMap::new()));
        assert!(client.send_message(invalid_msg).await.is_ok());
    }

    async fn send_valid_message(client: &mut Client) {
        let valid_msg = Message::from(Run::new("RETURN 1 as n;".to_string(), HashMap::new()));
        assert!(client.send_message(valid_msg).await.is_ok());
    }

    #[tokio::test]
    async fn handshake() {
        let client = new_client().await.unwrap();
        assert_eq!(client.version, 1);
    }

    #[tokio::test]
    async fn init_success() {
        let mut client = init_client("test").await.unwrap();
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn init_failure() {
        let mut client = init_client("invalid!").await.unwrap();
        let response = client.read_message().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ack_failure() {
        let mut client = get_initialized_client().await.unwrap();

        send_invalid_message(&mut client).await;
        let failure = Failure::try_from(client.read_message().await.unwrap());
        assert!(failure.is_ok());
        client.send_message(Message::AckFailure).await.unwrap();
        send_valid_message(&mut client).await;
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ack_failure_after_ignored() {
        let mut client = get_initialized_client().await.unwrap();

        send_invalid_message(&mut client).await;
        let failure = Failure::try_from(client.read_message().await.unwrap());
        assert!(failure.is_ok());

        send_valid_message(&mut client).await;
        let response = client.read_message().await.unwrap();
        assert!(match response {
            Message::Ignored => true,
            _ => false,
        });

        client.send_message(Message::AckFailure).await.unwrap();
        send_valid_message(&mut client).await;
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn run() {
        let mut client = get_initialized_client().await.unwrap();
        let run_msg = Message::from(Run::new("RETURN -1 as n;".to_string(), HashMap::new()));
        assert!(client.send_message(run_msg).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn run_and_pull() {
        let mut client = get_initialized_client().await.unwrap();
        let run_msg = Message::from(Run::new("RETURN 3458376 as n;".to_string(), HashMap::new()));
        assert!(client.send_message(run_msg).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        assert!(client.send_message(Message::PullAll).await.is_ok());
        let response = client.read_message().await.unwrap();
        let record = Record::try_from(response).unwrap();
        assert_eq!(record.fields(), &vec![Value::from(3458376)]);

        // After PullAll is finished, the server sends a Success message
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn discard_all_failure() {
        let mut client = get_initialized_client().await.unwrap();
        assert!(client.send_message(Message::DiscardAll).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn discard_all() {
        let mut client = get_initialized_client().await.unwrap();
        let run_msg = Message::from(Run::new("RETURN 3 as n;".to_string(), HashMap::new()));
        assert!(client.send_message(run_msg).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        assert!(client.send_message(Message::DiscardAll).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn discard_all_and_pull() {
        let mut client = get_initialized_client().await.unwrap();
        let run_msg = Message::from(Run::new("RETURN 3 as n;".to_string(), HashMap::new()));
        assert!(client.send_message(run_msg).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        assert!(client.send_message(Message::DiscardAll).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        assert!(client.send_message(Message::PullAll).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn reset() {
        let mut client = get_initialized_client().await.unwrap();
        let run_msg = Message::from(Run::new("Syntax error!;".to_string(), HashMap::new()));
        assert!(client.send_message(run_msg).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Failure::try_from(response).is_ok());

        send_valid_message(&mut client).await;
        let response = client.read_message().await.unwrap();
        assert!(match response {
            Message::Ignored => true,
            _ => false,
        });

        assert!(client.send_message(Message::Reset).await.is_ok());
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());

        send_valid_message(&mut client).await;
        let response = client.read_message().await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ignored() {
        let mut client = get_initialized_client().await.unwrap();
        send_invalid_message(&mut client).await;
        let failure = Failure::try_from(client.read_message().await.unwrap());
        assert!(failure.is_ok());
        send_valid_message(&mut client).await;

        let ignored = client.read_message().await.unwrap();
        assert!(match ignored {
            Message::Ignored => true,
            _ => false,
        });
    }
}
