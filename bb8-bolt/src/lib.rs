use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use bb8::ManageConnection;
use thiserror::Error;

use async_trait::async_trait;
use bolt_client::*;
use bolt_proto::*;

const SUPPORTED_VERSIONS: &[u32; 4] = &[4, 3, 2, 1];

pub struct BoltConnectionManager {
    addr: SocketAddr,
    domain: Option<String>,
    metadata: HashMap<String, Value>,
}

impl BoltConnectionManager {
    pub fn new(
        addr: impl ToSocketAddrs,
        domain: Option<String>,
        metadata: HashMap<String, impl Into<Value>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            addr: addr
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| Error::InvalidAddress)?,
            domain,
            metadata: metadata.into_iter().map(|(k, v)| (k, v.into())).collect(),
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Invalid host address.")]
    InvalidAddress,
    #[error("Invalid client version: {0}")]
    InvalidClientVersion(u32),
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
        client.handshake(SUPPORTED_VERSIONS).await?;
        let version = client.version().unwrap(); // ok to unwrap if handshake succeeds
        let response = match version {
            1 | 2 => {
                let mut metadata = self.metadata.clone();
                let user_agent = metadata.remove("user_agent").ok_or_else(|| {
                    Error::ClientInitFailed("metadata must contain a user_agent".to_string())
                })?;
                client.init(String::try_from(user_agent)?, metadata).await?
            }
            3 | 4 => client.hello(Metadata::from(self.metadata.clone())).await?,
            _ => return Err(Error::InvalidClientVersion(version)),
        };

        match response {
            Message::Success(_) => Ok(client),
            _ => Err(Error::ClientInitFailed(format!("{:?}", response))),
        }
    }

    async fn is_valid(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        let response = conn
            .run("RETURN 1;".to_string(), Default::default())
            .await?;
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
            HashMap::from_iter(vec![
                ("user_agent".to_string(), "bolt-client/X.Y.Z".to_string()),
                ("scheme".to_string(), "basic".to_string()),
                (
                    "principal".to_string(),
                    env::var("BOLT_TEST_USERNAME").unwrap(),
                ),
                (
                    "credentials".to_string(),
                    env::var("BOLT_TEST_PASSWORD").unwrap(),
                ),
            ]),
        )
        .unwrap()
    }

    #[tokio::test]
    // TODO: Because Neo4j picks the highest possible protocol version, this really only tests v3 or v4 clients. Find a
    //     way to test specific client versions. This may require refactoring that constant SUPPORTED_VERSIONS
    async fn basic_pool() {
        let manager = get_connection_manager();
        let pool = Pool::builder().max_size(15).build(manager).await.unwrap();

        let mut tasks = Vec::with_capacity(50);
        for i in 1..=tasks.capacity() {
            let pool = pool.clone();
            tasks.push(tokio::spawn(async move {
                let mut client = pool.get().await.unwrap();
                let statement = format!("RETURN {} as num;", i);
                let version = client.version().unwrap();
                let (response, records) = match version {
                    1 | 2 => {
                        client.run(statement, Default::default()).await.unwrap();
                        client.pull_all().await.unwrap()
                    }
                    3 => {
                        client
                            .run_with_metadata(statement, Default::default(), Default::default())
                            .await
                            .unwrap();
                        client.pull_all().await.unwrap()
                    }
                    4 => {
                        client
                            .run_with_metadata(statement, Default::default(), Default::default())
                            .await
                            .unwrap();
                        client
                            .pull(HashMap::from_iter(vec![("n".to_string(), 1)]))
                            .await
                            .unwrap()
                    }
                    _ => panic!("Unsupported client version: {}", version),
                };
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
            HashMap::from_iter(vec![
                ("user_agent".to_string(), "bolt-client/X.Y.Z"),
                ("scheme".to_string(), "basic"),
                ("principal".to_string(), "neo4j"),
                ("credentials".to_string(), "invalid"),
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
