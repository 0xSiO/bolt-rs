use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::IpAddr;

use bb8::ManageConnection;
use failure::{Error, Fail};

use async_trait::async_trait;
use bolt_client::message::Success;
use bolt_client::{Client, Message};

pub struct BoltConnectionManager {
    host: IpAddr,
    port: u16,
    client_name: String,
    auth_token: HashMap<String, String>,
}

impl BoltConnectionManager {
    pub fn new(
        host: IpAddr,
        port: u16,
        client_name: String,
        auth_token: HashMap<String, String>,
    ) -> Self {
        Self {
            host,
            port,
            client_name,
            auth_token,
        }
    }
}

#[derive(Debug, Fail)]
pub enum BoltConnectionError {
    #[fail(display = "Initialization of client failed: received {:?}", _0)]
    ClientInitFailed(Message),
}

#[async_trait]
impl ManageConnection for BoltConnectionManager {
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut client = Client::new((self.host, self.port)).await?;
        let response = client
            .init(self.client_name.clone(), self.auth_token.clone())
            .await?;
        if let Message::Success(_) = response {
            Ok(client)
        } else {
            Err(BoltConnectionError::ClientInitFailed(response).into())
        }
    }

    async fn is_valid(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        let response = conn.run("RETURN 1;".to_string(), None).await?;
        Success::try_from(response)?;
        let (response, _records) = conn.pull_all().await?;
        Success::try_from(response)?;
        Ok(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        // There's no good/fast way to check if a tokio TcpStream is still healthy. However, given that the TcpStream
        // is shut down when the connection object is dropped, we can assume existing connections aren't broken.
        false
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;
    use std::time::Duration;

    use bb8::*;

    use super::*;

    #[tokio::test]
    async fn basic_pool() {
        let manager = BoltConnectionManager::new(
            "127.0.0.1".parse().unwrap(),
            7687,
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (String::from("principal"), String::from("neo4j")),
                (String::from("credentials"), String::from("test")),
            ]),
        );
        let pool = Pool::builder().max_size(15).build(manager).await.unwrap();

        for i in 1..=30 {
            let pool = pool.clone();
            tokio::spawn(async move {
                let mut conn = pool.get().await.unwrap();
                let statement = format!("RETURN {} as num;", i);
                conn.run(statement, None).await.unwrap();
                let (response, records) = conn.pull_all().await.unwrap();
                assert!(Success::try_from(response).is_ok());
                assert_eq!(i32::try_from(records[0].fields()[0].clone()).unwrap(), i);
            });
        }
        tokio::time::delay_for(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn invalid_init_fails() {
        let manager = BoltConnectionManager::new(
            "127.0.0.1".parse().unwrap(),
            7687,
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (String::from("principal"), String::from("neo4j")),
                (String::from("credentials"), String::from("invalid")),
            ]),
        );
        let pool = Pool::builder().max_size(2).build(manager).await.unwrap();
        let conn = pool.dedicated_connection().await;
        assert!(conn.is_err());
    }
}
