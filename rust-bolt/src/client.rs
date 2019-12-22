use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::net::IpAddr;

use bytes::*;
use failure::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;

use crate::message::{Chunk, Init, Message, Success};
use crate::serialize::Serialize;
use std::sync::{Arc, Mutex};

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];
const SUPPORTED_VERSIONS: [u32; 4] = [1, 0, 0, 0];

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn new(host: IpAddr, port: usize) -> Result<Self, Error> {
        let client = Client {
            stream: TcpStream::connect(format!("{}:{}", host, port)).await?,
        };
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

    // TODO: Clean this up, this is just an experiment
    // Have to implement conversion from Bytes to value types before we can implement this
    pub async fn init(&mut self) -> Result<Success, Error> {
        println!("Starting init.");
        let init = Init::new(
            "rust-bolt/0.1.0",
            HashMap::from_iter(vec![
                ("scheme", "basic"),
                ("principal", "neo4j"),
                ("credentials", "test"),
            ]),
        );
        let bytes = init.try_into_bytes()?;
        let mut message = Message::with_capacity(bytes.len());
        message.add_chunk(Chunk::try_from(bytes)?);
        println!("Created message.");
        let mut bytes: Bytes = message.into();
        self.stream.write_buf(&mut bytes).await?;
        self.stream.flush().await?;
        println!("Wrote init.");
        // Success messages don't give us an EOF, so read exact number of bytes
        // TODO: To avoid this, we need to read on-demand from the stream: consider making Message read from
        //       a TcpStream instead of a Bytes (maybe even a Box<dyn AsyncBufRead> if that's possible).
        //       Remember to consume the last two 0 bytes when deserializing.
        let mut buf = vec![0u8; 27];
        self.stream.read_exact(&mut buf).await?;
        println!("Read response: {:?}", &buf[..]);
        let msg = Message::try_from(Bytes::from(buf))?;
        Ok(Success::try_from(Arc::new(Mutex::new(msg.bytes.freeze())))?)
    }
}
