use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use async_trait::async_trait;
use bb8::ManageConnection;
use thiserror::Error;

use bolt_client::*;
use bolt_proto::*;

pub struct BoltConnectionManager {
    addr: SocketAddr,
    domain: Option<String>,
    supported_versions: [u32; 4],
    metadata: HashMap<String, Value>,
}

impl BoltConnectionManager {
    pub fn new(
        addr: impl ToSocketAddrs,
        domain: Option<String>,
        supported_versions: [u32; 4],
        metadata: HashMap<impl Into<String>, impl Into<Value>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            addr: addr
                .to_socket_addrs()?
                .next()
                .ok_or_else(|| Error::InvalidAddress)?,
            domain,
            supported_versions,
            metadata: metadata
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("invalid host address.")]
    InvalidAddress,
    #[error("invalid client version: {0:#x}")]
    InvalidClientVersion(u32),
    #[error("initialization of client failed: {0}")]
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
        let mut client = Client::new(self.addr, self.domain.as_ref()).await?;
        let version = client.handshake(&self.supported_versions).await?;
        let response = match version {
            1 | 2 => {
                let mut metadata = self.metadata.clone();
                let user_agent: String = metadata
                    .remove("user_agent")
                    .ok_or_else(|| {
                        Error::ClientInitFailed("metadata must contain a user_agent".to_string())
                    })
                    .map(String::try_from)??;
                client.init(user_agent, Metadata::from(metadata)).await?
            }
            3 | 4 | 0x0104 => {
                client
                    .hello(Some(Metadata::from(self.metadata.clone())))
                    .await?
            }
            _ => return Err(Error::InvalidClientVersion(version)),
        };

        match response {
            Message::Success(_) => Ok(client),
            _ => Err(Error::ClientInitFailed(format!("{:?}", response))),
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
    use futures_util::future::join_all;

    use super::*;

    fn get_connection_manager(supported_versions: [u32; 4]) -> BoltConnectionManager {
        BoltConnectionManager::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok(),
            supported_versions,
            HashMap::from_iter(vec![
                ("user_agent", "bolt-client/X.Y.Z"),
                ("scheme", "basic"),
                ("principal", &env::var("BOLT_TEST_USERNAME").unwrap()),
                ("credentials", &env::var("BOLT_TEST_PASSWORD").unwrap()),
            ]),
        )
        .unwrap()
    }

    async fn is_server_compatible(bolt_version: u32) -> Result<bool, Error> {
        let mut client = Client::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok(),
        )
        .await?;
        Ok(client.handshake(&[bolt_version, 0, 0, 0]).await.is_ok())
    }

    #[tokio::test]
    async fn basic_pool() {
        for &bolt_version in &[1_u32, 2, 3, 4, 0x0104] {
            // Don't even test connection pool if server doesn't support this Bolt version
            if !is_server_compatible(bolt_version).await.unwrap() {
                println!(
                    "Skipping test: server doesn't support Bolt version {:#x}.",
                    bolt_version
                );
                continue;
            }

            let manager = get_connection_manager([bolt_version, 0, 0, 0]);
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
                            client.run(statement, None).await.unwrap();
                            client.pull_all().await.unwrap()
                        }
                        3 => {
                            client
                                .run_with_metadata(statement, None, None)
                                .await
                                .unwrap();
                            client.pull_all().await.unwrap()
                        }
                        4 | 0x0104 => {
                            client
                                .run_with_metadata(statement, None, None)
                                .await
                                .unwrap();
                            client
                                .pull(Some(Metadata::from_iter(vec![("n".to_string(), 1)])))
                                .await
                                .unwrap()
                        }
                        _ => panic!("Unsupported client version: {:#x}", version),
                    };
                    assert!(message::Success::try_from(response).is_ok());
                    assert_eq!(records[0].fields(), &[Value::from(i as i8)]);
                }));
            }
            join_all(tasks).await;
        }
    }

    #[tokio::test]
    async fn invalid_init_fails() {
        let invalid_manager = BoltConnectionManager::new(
            "127.0.0.1:7687",
            None,
            [4, 3, 2, 1],
            HashMap::from_iter(vec![
                ("user_agent", "bolt-client/X.Y.Z"),
                ("scheme", "basic"),
                ("principal", "neo4j"),
                ("credentials", "invalid"),
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
