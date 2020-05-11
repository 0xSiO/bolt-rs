use std::collections::HashMap;

use bolt_client_macros::*;
use bolt_proto::message::*;
use bolt_proto::{Message, Value};

use crate::error::*;
use crate::Client;

// TODO: Confirm behavior of new messages, test it and document it

impl Client {
    /// Send a `DISCARD` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `DISCARD_ALL` for Bolt v3+ clients, but allows passing an arbitrary metadata
    /// hash along with the request.
    #[bolt_version(4)]
    pub async fn discard(
        &mut self,
        metadata: HashMap<String, impl Into<Value>>,
    ) -> Result<Message> {
        let discard_msg = Discard::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Discard(discard_msg)).await?;
        self.read_message().await
    }

    /// Send a `PULL` message to the server.
    ///
    /// # Description
    /// This message is the equivalent of `PULL_ALL` for Bolt v3+ clients, but allows passing an arbitrary metadata hash
    /// along with the request.
    #[bolt_version(4)]
    pub async fn pull(&mut self, metadata: HashMap<String, impl Into<Value>>) -> Result<Message> {
        let pull_msg = Pull::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Pull(pull_msg)).await?;
        self.read_message().await
    }
}
