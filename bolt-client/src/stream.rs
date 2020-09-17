use std::{
    fmt::Debug,
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::io::{AsyncRead, AsyncWrite};
use pin_project::pin_project;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_util::compat::Compat;

#[pin_project(project = StreamProj)]
#[derive(Debug)]
pub(crate) enum Stream {
    Tcp(#[pin] Compat<TcpStream>),
    SecureTcp(#[pin] Compat<TlsStream<TcpStream>>),
}

impl AsyncRead for Stream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => AsyncRead::poll_read(tcp_stream, cx, buf),
            StreamProj::SecureTcp(tls_stream) => AsyncRead::poll_read(tls_stream, cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => AsyncWrite::poll_write(tcp_stream, cx, buf),
            StreamProj::SecureTcp(tls_stream) => AsyncWrite::poll_write(tls_stream, cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => AsyncWrite::poll_flush(tcp_stream, cx),
            StreamProj::SecureTcp(tls_stream) => AsyncWrite::poll_flush(tls_stream, cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => AsyncWrite::poll_close(tcp_stream, cx),
            StreamProj::SecureTcp(tls_stream) => AsyncWrite::poll_close(tls_stream, cx),
        }
    }
}
