// Much of the documentation comments for message-related instance methods in Client and
// its submodules are copied from the descriptions given by Neo Technology, Inc. on
// https://boltprotocol.org/v1/, with minor modifications.
//
// The aforementioned comments are thus licensed under the Creative Commons
// Attribution-ShareAlike 3.0 Unported License. To view a copy of this license, visit
// http://creativecommons.org/licenses/by-sa/3.0/ or send a letter to Creative Commons,
// PO Box 1866, Mountain View, CA 94042, USA.

use std::convert::TryInto;

use bytes::*;
use futures_util::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use bolt_proto::Message;

use crate::error::*;

mod v1;
mod v2;
mod v3;
mod v4;
mod v4_1;

const PREAMBLE: [u8; 4] = [0x60, 0x60, 0xB0, 0x17];

/// An asynchronous client for Bolt servers.
#[derive(Debug)]
pub struct Client<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
    version: u32,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Client<S> {
    /// Attempt to create a new client from an asynchronous stream. A handshake will be
    /// performed with the provided protocol versions, and, if this succeeds, a Client will be
    /// returned.
    pub async fn new(mut stream: S, preferred_versions: &[u32; 4]) -> Result<Self> {
        let mut preferred_versions_bytes = BytesMut::with_capacity(16);
        preferred_versions
            .iter()
            .for_each(|&v| preferred_versions_bytes.put_u32(v));
        stream.write_all(&PREAMBLE).await?;
        stream.write_all(&preferred_versions_bytes).await?;
        stream.flush().await?;

        let mut u32_bytes = [0, 0, 0, 0];
        stream.read_exact(&mut u32_bytes).await?;
        let version = u32::from_be_bytes(u32_bytes);
        if preferred_versions.contains(&version) && version > 0 {
            Ok(Self { stream, version })
        } else {
            Err(Error::HandshakeFailed(*preferred_versions))
        }
    }

    /// Get the current version of this client.
    pub fn version(&self) -> u32 {
        self.version
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
        for chunk in chunks {
            self.stream.write_all(&chunk).await?;
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
            for chunk in chunks {
                self.stream.write_all(&chunk).await?;
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
