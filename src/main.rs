use std::error::Error;

use bytes::*;
use tokio::net::TcpStream;
use tokio::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:7687").await?;
    let chosen_version = handshake(&mut stream).await?;
    println!("Chosen version: {}", chosen_version);

    Ok(())
}

async fn handshake(stream: &mut TcpStream) -> Result<u32, Box<dyn Error>> {
    const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];
    const ALLOWED_VERSIONS: [u32; 4] = [1, 0, 0, 0];

    let mut allowed_versions = BytesMut::with_capacity(16);
    ALLOWED_VERSIONS.iter().for_each(|&v| allowed_versions.put_u32(v));

    stream.write(&mut PREAMBLE).await?;
    stream.write_buf(&mut allowed_versions).await?;
    stream.flush().await?;

    Ok(stream.read_u32().await?)
}
