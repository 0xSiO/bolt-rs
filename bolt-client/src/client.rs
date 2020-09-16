// Much of the documentation comments for message-related instance methods in Client and
// its submodules are copied from the descriptions given by Neo Technology, Inc. on
// https://boltprotocol.org/v1/, with minor modifications.
//
// The aforementioned comments are thus licensed under the Creative Commons
// Attribution-ShareAlike 3.0 Unported License. To view a copy of this license, visit
// http://creativecommons.org/licenses/by-sa/3.0/ or send a letter to Creative Commons,
// PO Box 1866, Mountain View, CA 94042, USA.

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
mod v2;
mod v3;
mod v4;
mod v4_1;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];

/// A tokio-based client for Bolt servers.
#[derive(Debug)]
pub struct Client {
    stream: BufStream<Stream>,
    version: Option<u32>,
}

impl Client {
    /// Create a new client pointing to the provided server address. If a server domain
    /// is provided, the client will attempt to connect to the server over a connection
    /// secured with TLS.
    pub async fn new(addr: impl ToSocketAddrs, domain: Option<impl Into<String>>) -> Result<Self> {
        let stream = match domain {
            Some(domain) => {
                let domain = domain.into();
                let tls_connector = Client::configure_tls_connector(&TLS_SERVER_ROOTS);
                let dns_name_ref = DNSNameRef::try_from_ascii_str(&domain)
                    .map_err(|_| Error::InvalidDNSName(domain.clone()))?;
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

    /// Get the current version of this client.
    pub fn version(&self) -> &Option<u32> {
        &self.version
    }

    fn configure_tls_connector(root_certs: &webpki::TLSServerTrustAnchors) -> TlsConnector {
        let mut config = ClientConfig::new();
        config.root_store.add_server_trust_anchors(root_certs);
        TlsConnector::from(Arc::new(config))
    }

    /// Perform a handshake with the Bolt server and agree upon a protocol version to
    /// use for the client. Returns the version that was agreed upon.
    pub async fn handshake(&mut self, preferred_versions: &[u32; 4]) -> Result<u32> {
        let mut preferred_versions_bytes = BytesMut::with_capacity(16);
        preferred_versions
            .iter()
            .for_each(|&v| preferred_versions_bytes.put_u32(v));
        self.stream.write(&PREAMBLE).await?;
        self.stream.write_buf(&mut preferred_versions_bytes).await?;
        self.stream.flush().await?;

        let version: u32 = self.stream.read_u32().await?;
        if preferred_versions.contains(&version) && version > 0 {
            self.version = Some(version);
            Ok(version)
        } else {
            Err(Error::HandshakeFailed(*preferred_versions))
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

    /// Send multiple messages to the server without waiting for a response. Returns a
    /// [`Vec`] containing the server's response messages for each of the sent messages,
    /// in the order they were provided.
    ///
    /// # Description
    /// The client is not required to wait for a response before sending more messages.
    /// Sending multiple messages together like this is called pipelining. For performance
    /// reasons, it is recommended that clients use pipelining as much as possible.
    /// Through pipelining, multiple messages can be transmitted together in the same
    /// network package, significantly reducing latency and increasing throughput.
    ///
    /// A common technique is to buffer outgoing messages on the client until the last
    /// possible moment, such as when a commit is issued or a result is read by the
    /// application, and then sending all messages in the buffer together.
    ///
    /// # Failure Handling
    /// Because the protocol leverages pipelining, the client and the server need to agree
    /// on what happens when a failure occurs, otherwise messages that were sent assuming
    /// no failure would occur might have unintended effects.
    ///
    /// When requests fail on the server, the server will send the client a `FAILURE`
    /// message. The client must acknowledge the `FAILURE` message by sending a `RESET`
    /// (Bolt v3+) or `ACK_FAILURE` (Bolt v1-2) message to the server. Until the server
    /// receives the `RESET`/`ACK_FAILURE` message, it will send an `IGNORED` message in
    /// response to any other message from the client, including messages that were sent
    /// in a pipeline.
    pub async fn pipeline(&mut self, messages: Vec<Message>) -> Result<Vec<Message>> {
        // This Vec is too small if we're expecting some RECORD messages, so there's no "good" size
        let mut responses = Vec::with_capacity(messages.len());

        for message in messages {
            #[cfg(test)]
            println!(">>> {:?}", message);

            let chunks: Vec<Bytes> = message.try_into()?;
            for mut chunk in chunks {
                self.stream.write_buf(&mut chunk).await?;
            }
        }
        self.stream.flush().await?;

        for _ in 0..responses.capacity() {
            let mut response = self.read_message().await?;
            while let Message::Record(_) = response {
                responses.push(response);
                response = self.read_message().await?;
            }
            responses.push(response);
        }
        Ok(responses)
    }
}
