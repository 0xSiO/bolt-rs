use std::collections::HashMap;

use bolt_client_macros::*;
use bolt_proto::message::*;
use bolt_proto::{Message, Value};

use crate::error::*;
use crate::Client;

// TODO: Test the rest of the new v3 messages to determine request/response behavior
impl Client {
    #[bolt_version(3, 4)]
    pub async fn hello(&mut self, metadata: HashMap<String, impl Into<Value>>) -> Result<Message> {
        let hello_msg = Hello::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Hello(hello_msg)).await?;
        self.read_message().await
    }

    // Closes connection to server, no message sent in response
    #[bolt_version(3, 4)]
    pub async fn goodbye(&mut self) -> Result<()> {
        self.send_message(Message::Goodbye).await?;
        Ok(())
    }

    #[bolt_version(3, 4)]
    pub async fn run_with_metadata(
        &mut self,
        statement: String,
        parameters: Option<HashMap<String, Value>>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<Message> {
        let run_msg = RunWithMetadata::new(
            statement,
            parameters.unwrap_or_default(),
            metadata.unwrap_or_default(),
        );
        self.send_message(Message::RunWithMetadata(run_msg)).await?;
        self.read_message().await
    }

    #[bolt_version(3, 4)]
    pub async fn begin(&mut self, metadata: HashMap<String, impl Into<Value>>) -> Result<Message> {
        let begin_msg = Begin::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Begin(begin_msg)).await?;
        // TODO: Is there actually a response?
        self.read_message().await
    }

    #[bolt_version(3, 4)]
    pub async fn commit(&mut self) -> Result<Message> {
        self.send_message(Message::Commit).await?;
        // TODO: Is there actually a response?
        self.read_message().await
    }

    #[bolt_version(3, 4)]
    pub async fn rollback(&mut self) -> Result<Message> {
        self.send_message(Message::Rollback).await?;
        // TODO: Is there actually a response?
        self.read_message().await
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::client::v1::tests::*;
    use crate::skip_if_handshake_failed;

    use super::*;

    #[tokio::test]
    async fn hello() {
        let client = new_client(3).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn hello_fail() {
        let client = new_client(3).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn goodbye() {
        let client = get_initialized_client(3).await;
        skip_if_handshake_failed!(client);
        let mut client = client.unwrap();
        assert!(client.goodbye().await.is_ok());
    }
}
