use std::convert::TryInto;
use std::sync::Arc;

use bytes::*;
use tokio::io::BufStream;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::prelude::*;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::webpki::DNSNameRef;
use tokio_rustls::{webpki, TlsConnector};
use webpki_roots::TLS_SERVER_ROOTS;

use bolt_proto::Message;

use crate::error::*;
use crate::stream::Stream;

mod v1;
mod v3;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];

#[derive(Debug)]
pub struct Client {
    pub(crate) stream: BufStream<Stream>,
    pub(crate) version: Option<u32>,
}

impl Client {
    /// Create a new client pointing to the provided server address. If a server domain is provided, the Client will
    /// attempt to connect to the server over a connection secured with TLS.
    pub async fn new(addr: impl ToSocketAddrs, domain: Option<&str>) -> Result<Self> {
        let stream = match domain {
            Some(domain) => {
                let tls_connector = Client::configure_tls_connector(&TLS_SERVER_ROOTS);
                let dns_name_ref = DNSNameRef::try_from_ascii_str(&domain)
                    .map_err(|_| Error::InvalidDNSName(domain.to_string()))?;
                let stream = TcpStream::connect(addr).await?;
                Stream::SecureTcp(Box::new(tls_connector.connect(dns_name_ref, stream).await?))
            }
            None => Stream::Tcp(TcpStream::connect(addr).await?),
        };
        Ok(Client {
            stream: BufStream::new(stream),
            version: None,
        })
    }

    fn configure_tls_connector(root_certs: &webpki::TLSServerTrustAnchors) -> TlsConnector {
        let mut config = ClientConfig::new();
        config.root_store.add_server_trust_anchors(root_certs);
        TlsConnector::from(Arc::new(config))
    }

    /// Perform a handshake with the Bolt server and agree upon a protocol version to use for the client.
    pub async fn handshake(&mut self, supported_versions: &[u32; 4]) -> Result<()> {
        let mut allowed_versions = BytesMut::with_capacity(16);
        supported_versions
            .iter()
            .for_each(|&v| allowed_versions.put_u32(v));
        self.stream.write(&PREAMBLE).await?;
        self.stream.write_buf(&mut allowed_versions).await?;
        self.stream.flush().await?;

        let version = self.stream.read_u32().await?;
        if supported_versions.contains(&version) && version > 0 {
            self.version = Some(version);
            Ok(())
        } else {
            Err(Error::HandshakeFailed)
        }
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
