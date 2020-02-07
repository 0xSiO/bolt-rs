use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use bb8::ManageConnection;
use failure::{Error, Fail};

use async_trait::async_trait;
use bolt_client::*;
use bolt_proto::v1::*;

pub struct BoltV1ConnectionManager {
    addr: SocketAddr,
    domain: Option<String>,
    client_name: String,
    auth_token: HashMap<String, String>,
}

impl BoltV1ConnectionManager {
    pub fn new(
        addr: impl ToSocketAddrs,
        domain: Option<String>,
        client_name: String,
        auth_token: HashMap<String, String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            addr: addr
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| BoltConnectionError::InvalidAddress)?,
            domain,
            client_name,
            auth_token,
        })
    }
}

#[derive(Debug, Fail)]
pub enum BoltConnectionError {
    #[fail(display = "Failed to connect: invalid host address.")]
    InvalidAddress,
    #[fail(display = "Initialization of client failed: {}", _0)]
    ClientInitFailed(String),
}

#[async_trait]
impl ManageConnection for BoltV1ConnectionManager {
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut client = Client::new(self.addr, self.domain.as_deref()).await?;
        let response = client
            .init(self.client_name.clone(), self.auth_token.clone())
            .await?;
        if let Message::Success(_) = response {
            Ok(client)
        } else {
            Err(BoltConnectionError::ClientInitFailed(format!("{:?}", response)).into())
        }
    }

    async fn is_valid(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        let response = conn.run("RETURN 1;".to_string(), None).await?;
        message::Success::try_from(response)?;
        let (response, _records) = conn.pull_all().await?;
        message::Success::try_from(response)?;
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
    use std::env;
    use std::iter::FromIterator;

    use bb8::*;

    use super::*;

    fn get_connection_manager() -> BoltV1ConnectionManager {
        BoltV1ConnectionManager::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok(),
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (
                    String::from("principal"),
                    env::var("BOLT_TEST_USERNAME").unwrap(),
                ),
                (
                    String::from("credentials"),
                    env::var("BOLT_TEST_PASSWORD").unwrap(),
                ),
            ]),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn basic_pool() {
        let manager = get_connection_manager();
        let pool = Pool::builder().max_size(15).build(manager).await.unwrap();

        let mut tasks = Vec::with_capacity(50);
        for i in 1..=tasks.capacity() {
            let pool = pool.clone();
            tasks.push(tokio::spawn(async move {
                let mut conn = pool.get().await.unwrap();
                let statement = format!("RETURN {} as num;", i);
                conn.run(statement, None).await.unwrap();
                let (response, records) = conn.pull_all().await.unwrap();
                assert!(message::Success::try_from(response).is_ok());
                assert_eq!(records[0].fields(), &[Value::from(i as i8)]);
            }));
        }
        tokio::join!(futures::future::join_all(tasks));
    }

    #[tokio::test]
    async fn invalid_init_fails() {
        let invalid_manager = BoltV1ConnectionManager::new(
            "127.0.0.1:7687",
            None,
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (String::from("principal"), String::from("neo4j")),
                (String::from("credentials"), String::from("invalid")),
            ]),
        )
        .unwrap();
        let pool = Pool::builder()
            .max_size(2)
            .build(invalid_manager)
            .await
            .unwrap();
        let conn = pool.dedicated_connection().await;
        assert!(conn.is_err());
    }
}
