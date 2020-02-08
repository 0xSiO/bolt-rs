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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use std::env;
    use std::iter::FromIterator;

    use bolt_proto::message::*;

    use super::*;

    async fn new_client() -> Result<Client> {
        let client = Client::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok().as_deref(),
        )
        .await?;
        assert_eq!(client.version, 4);
        Ok(client)
    }

    async fn initialize_client(client: &mut Client, succeed: bool) -> Result<Message> {
        let username = env::var("BOLT_TEST_USERNAME").unwrap();
        let password = if succeed {
            env::var("BOLT_TEST_PASSWORD").unwrap()
        } else {
            "invalid".to_string()
        };

        client
            .hello(HashMap::from_iter(vec![
                (String::from("user_agent"), "bolt-client/X.Y.Z"),
                (String::from("scheme"), "basic"),
                (String::from("principal"), &username),
                (String::from("credentials"), &password),
            ]))
            .await
    }

    #[tokio::test]
    async fn hello() {
        let mut client = new_client().await.unwrap();
        let response = initialize_client(&mut client, true).await.unwrap();
        assert!(Success::try_from(response).is_ok());
    }
}
