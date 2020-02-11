use std::fmt::Debug;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_rustls::client::TlsStream;

#[derive(Debug)]
pub(crate) enum Stream {
    Tcp(TcpStream),
    SecureTcp(TlsStream<TcpStream>),
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::result::Result<usize, io::Error>> {
        match self.get_mut() {
            Stream::Tcp(tcp_stream) => Pin::new(tcp_stream).poll_read(cx, buf),
            Stream::SecureTcp(tls_stream) => Pin::new(tls_stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            Stream::Tcp(tcp_stream) => Pin::new(tcp_stream).poll_write(cx, buf),
            Stream::SecureTcp(tls_stream) => Pin::new(tls_stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Stream::Tcp(tcp_stream) => Pin::new(tcp_stream).poll_flush(cx),
            Stream::SecureTcp(tls_stream) => Pin::new(tls_stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Stream::Tcp(tcp_stream) => Pin::new(tcp_stream).poll_shutdown(cx),
            Stream::SecureTcp(tls_stream) => Pin::new(tls_stream).poll_shutdown(cx),
        }
    }
}
