use std::error::Error;
use std::net::IpAddr;

use bytes::*;
use tokio::net::TcpStream;
use tokio::prelude::*;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];
const SUPPORTED_VERSIONS: [u32; 4] = [1, 0, 0, 0];

pub struct BoltClient {
    stream: TcpStream
}

impl BoltClient {
    pub async fn new(host: IpAddr, port: usize) -> Result<Self, Box<dyn Error>> {
        let client = BoltClient {
            stream: TcpStream::connect(format!("{}:{}", host, port)).await?
        };
        Ok(client)
    }

    pub async fn handshake(&mut self) -> Result<u32, Box<dyn Error>> {
        let mut allowed_versions = BytesMut::with_capacity(16);
        SUPPORTED_VERSIONS.iter().for_each(|&v| allowed_versions.put_u32(v));
        self.stream.write(&mut PREAMBLE).await?;
        self.stream.write_buf(&mut allowed_versions).await?;
        self.stream.flush().await?;
        Ok(self.stream.read_u32().await?)
    }
}
