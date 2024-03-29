use std::{
    fmt::Debug,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpStream, ToSocketAddrs},
};
use tokio_rustls::{
    client::TlsStream,
    rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName},
    TlsConnector,
};

/// A convenient wrapper around a [`TcpStream`](tokio::net::TcpStream) or a
/// [`TlsStream`](tokio_rustls::client::TlsStream).
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-stream")))]
#[pin_project(project = StreamProj)]
#[derive(Debug)]
pub enum Stream {
    Tcp(#[pin] TcpStream),
    SecureTcp(#[pin] Box<TlsStream<TcpStream>>),
}

impl Stream {
    /// Establish a connection with a remote socket. If a domain is provided, TLS negotiation will
    /// be attempted.
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-stream")))]
    pub async fn connect(
        addr: impl ToSocketAddrs,
        domain: Option<impl AsRef<str>>,
    ) -> io::Result<Self> {
        match domain {
            Some(domain) => {
                let mut root_cert_store = RootCertStore::empty();
                root_cert_store.add_server_trust_anchors(
                    webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|anchor| {
                        OwnedTrustAnchor::from_subject_spki_name_constraints(
                            anchor.subject,
                            anchor.spki,
                            anchor.name_constraints,
                        )
                    }),
                );

                let config = ClientConfig::builder()
                    .with_safe_defaults()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();

                let server_name = ServerName::try_from(domain.as_ref())
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, domain.as_ref()))?;

                let stream = TcpStream::connect(addr).await?;

                Ok(Stream::SecureTcp(Box::new(
                    TlsConnector::from(Arc::new(config))
                        .connect(server_name, stream)
                        .await?,
                )))
            }
            None => Ok(Stream::Tcp(TcpStream::connect(addr).await?)),
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => tcp_stream.poll_read(cx, buf),
            StreamProj::SecureTcp(tls_stream) => tls_stream.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => tcp_stream.poll_write(cx, buf),
            StreamProj::SecureTcp(tls_stream) => tls_stream.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => tcp_stream.poll_flush(cx),
            StreamProj::SecureTcp(tls_stream) => tls_stream.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => tcp_stream.poll_shutdown(cx),
            StreamProj::SecureTcp(tls_stream) => tls_stream.poll_shutdown(cx),
        }
    }
}
