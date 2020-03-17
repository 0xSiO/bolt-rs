use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use bb8::ManageConnection;
use thiserror::Error;

use async_trait::async_trait;
use bolt_client::*;
use bolt_proto::*;

pub struct BoltConnectionManager {
    addr: SocketAddr,
    domain: Option<String>,
    client_name: String,
    auth_token: HashMap<String, Value>,
}

impl BoltConnectionManager {
    pub fn new(
        addr: impl ToSocketAddrs,
        domain: Option<String>,
        client_name: String,
        auth_token: HashMap<String, Value>,
    ) -> Result<Self, Error> {
        Ok(Self {
            addr: addr
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| Error::InvalidAddress)?,
            domain,
            client_name,
            auth_token,
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Invalid host address.")]
    InvalidAddress,
    #[error("Initialization of client failed: {0}")]
    ClientInitFailed(String),
    #[error(transparent)]
    ClientError(#[from] bolt_client::error::Error),
    #[error(transparent)]
    ProtocolError(#[from] bolt_proto::error::Error),
}

#[async_trait]
impl ManageConnection for BoltConnectionManager {
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut client = Client::new(self.addr, self.domain.as_deref()).await?;
        client.handshake(&[1, 0, 0, 0]).await?; // TODO: Update this to use higher versions when implemented
        let response = client
            .init(self.client_name.clone(), self.auth_token.clone())
            .await?;
        if let Message::Success(_) = response {
            Ok(client)
        } else {
            Err(Error::ClientInitFailed(format!("{:?}", response)))
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

    fn get_connection_manager() -> BoltConnectionManager {
        BoltConnectionManager::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok(),
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), Value::from("basic")),
                (
                    String::from("principal"),
                    Value::from(env::var("BOLT_TEST_USERNAME").unwrap()),
                ),
                (
                    String::from("credentials"),
                    Value::from(env::var("BOLT_TEST_PASSWORD").unwrap()),
                ),
            ]),
        )
        .unwrap()
    }

    #[tokio::test]
    #[allow(unreachable_code)]
    async fn basic_pool() {
        // TODO: Support the newer client versions
        println!("Skipping test: need to support newer client versions.");
        return;

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
        let invalid_manager = BoltConnectionManager::new(
            "127.0.0.1:7687",
            None,
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), Value::from("basic")),
                (String::from("principal"), Value::from("neo4j")),
                (String::from("credentials"), Value::from("invalid")),
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
