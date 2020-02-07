use std::convert::TryInto;

use bytes::*;
use tokio::io::BufStream;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::prelude::*;

use bolt_proto::Message;

use crate::error::*;
use crate::stream::Stream;

mod v1;
mod v3;

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
                async_native_tls::connect(&domain, TcpStream::connect(addr).await?).await?,
            ),
            None => Stream::Tcp(TcpStream::connect(addr).await?),
        };
        let mut client = Client {
            stream: BufStream::new(stream),
            version: 0,
        };
        client.version = client.handshake().await? as u8;

        if client.version == 1 {
            Ok(client)
        } else {
            Err(ClientError::UnsupportedClientVersion(client.version).into())
        }
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
}
