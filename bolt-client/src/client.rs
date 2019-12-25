use std::convert::TryInto;
use std::net::IpAddr;

use bytes::*;
use failure::Error;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use tokio::prelude::*;

use bolt_proto::Message;

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

    pub async fn handshake(&mut self) -> Result<u32, Error> {
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::iter::FromIterator;

    use bolt_proto::message::{Failure, Init, Success};
    use bolt_proto::Value;

    use super::*;

    async fn new_client() -> Result<Client, Error> {
        Client::new("127.0.0.1".parse().unwrap(), 7687).await
    }

    async fn init_client(credentials: &str) -> Result<Client, Error> {
        let mut client = new_client().await?;
        assert!(client
            .send_message(Message::from(Init::new(
                "bolt-client/0.2.0".to_string(),
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

    #[tokio::test]
    async fn handshake() {
        let client = new_client().await.unwrap();
        assert_eq!(client.version, 1);
    }

    #[tokio::test]
    async fn init_success() {
        let mut client = init_client("test").await.unwrap();
        let response = client.read_message().await.unwrap();
        // println!("{:?}", response);
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn init_failure() {
        let mut client = init_client("invalid!").await.unwrap();
        let response = client.read_message().await.unwrap();
        // println!("{:?}", response);
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn ack_failure() {
        todo!();
    }

    #[tokio::test]
    async fn run() {
        todo!();
    }

    #[tokio::test]
    async fn discard_all() {
        todo!();
    }

    #[tokio::test]
    async fn pull_all() {
        todo!();
    }

    #[tokio::test]
    async fn reset() {
        todo!();
    }

    #[tokio::test]
    async fn ignored() {
        todo!();
    }
}
