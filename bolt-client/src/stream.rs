use std::{
    fmt::Debug,
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::client::TlsStream;

#[pin_project(project = StreamProj)]
#[derive(Debug)]
pub enum Stream {
    Tcp(#[pin] TcpStream),
    SecureTcp(#[pin] TlsStream<TcpStream>),
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

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        match self.project() {
            StreamProj::Tcp(tcp_stream) => AsyncWrite::poll_shutdown(tcp_stream, cx),
            StreamProj::SecureTcp(tls_stream) => AsyncWrite::poll_shutdown(tls_stream, cx),
        }
    }
}
