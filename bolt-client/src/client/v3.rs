use std::collections::HashMap;

use bolt_client_macros::*;
use bolt_proto::message::*;
use bolt_proto::{Message, Value};

use crate::error::*;
use crate::Client;

impl Client {
    #[bolt_version(3, 4)]
    pub async fn hello(&mut self, metadata: HashMap<String, impl Into<Value>>) -> Result<Message> {
        let hello_msg = Hello::new(metadata.into_iter().map(|(k, v)| (k, v.into())).collect());
        self.send_message(Message::Hello(hello_msg)).await?;
        self.read_message().await
    }

    // TODO: Implement run_with_metadata, or just modify run if possible
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::client::v1::tests::*;
    use crate::compatible_versions;

    use super::*;

    #[tokio::test]
    async fn hello() {
        let mut client = new_client().await.unwrap();
        compatible_versions!(client, 3, 4);
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }

    #[tokio::test]
    async fn hello_fail() {
        let mut client = new_client().await.unwrap();
        compatible_versions!(client, 3, 4);
        let response = initialize_client(&mut client, false).await.unwrap();
        assert!(Failure::try_from(response).is_ok());
    }
}
