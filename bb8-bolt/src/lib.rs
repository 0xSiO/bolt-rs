use std::{collections::HashMap, convert::TryFrom, net::SocketAddr};

use async_trait::async_trait;
use bb8::{ManageConnection, PooledConnection};
use thiserror::Error;
use tokio::{
    io::BufStream,
    net::{lookup_host, ToSocketAddrs},
};
use tokio_util::compat::*;

use bolt_client::*;
use bolt_proto::{version::*, *};

pub use bolt_client;
pub use bolt_proto;

pub struct BoltConnectionManager {
    addr: SocketAddr,
    domain: Option<String>,
    preferred_versions: [u32; 4],
    metadata: HashMap<String, Value>,
}

impl BoltConnectionManager {
    pub async fn new(
        addr: impl ToSocketAddrs,
        domain: Option<String>,
        preferred_versions: [u32; 4],
        metadata: HashMap<impl Into<String>, impl Into<Value>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            addr: lookup_host(addr)
                .await?
                .next()
                .ok_or(Error::InvalidAddress)?,
            domain,
            preferred_versions,
            metadata: metadata
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid host address")]
    InvalidAddress,
    #[error("invalid metadata: {0}")]
    InvalidMetadata(String),
    #[error("client initialization failed: received {0:?}")]
    ClientInitFailed(bolt_proto::Message),
    #[error("invalid client version: {0:#x}")]
    InvalidClientVersion(u32),
    #[error(transparent)]
    ClientError(#[from] bolt_client::error::Error),
    #[error(transparent)]
    ProtocolError(#[from] bolt_proto::error::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[async_trait]
impl ManageConnection for BoltConnectionManager {
    type Connection = Client<Compat<BufStream<Stream>>>;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut client = Client::new(
            BufStream::new(Stream::connect(self.addr, self.domain.as_ref()).await?).compat(),
            &self.preferred_versions,
        )
        .await?;
        let response = match client.version() {
            V1_0 | V2_0 => {
                let mut metadata = self.metadata.clone();
                let user_agent: String = metadata
                    .remove("user_agent")
                    .ok_or_else(|| Error::InvalidMetadata("must contain a user_agent".to_string()))
                    .map(String::try_from)??;
                client.init(user_agent, Metadata::from(metadata)).await?
            }
            V3_0 | V4_0 | V4_1 => {
                client
                    .hello(Some(Metadata::from(self.metadata.clone())))
                    .await?
            }
            _ => return Err(Error::InvalidClientVersion(client.version())),
        };

        match response {
            Message::Success(_) => Ok(client),
            other => Err(Error::ClientInitFailed(other)),
        }
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        let response = conn.run("RETURN 1;".to_string(), None).await?;
        message::Success::try_from(response)?;
        let (response, _records) = conn.pull_all().await?;
        message::Success::try_from(response)?;
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        // There's no good/fast way to check if a tokio TcpStream is still healthy.
        // However, given that the TcpStream is shut down when the connection object is
        // dropped, we can assume existing connections aren't broken.
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

    async fn get_connection_manager(
        preferred_versions: [u32; 4],
        succeed: bool,
    ) -> BoltConnectionManager {
        let credentials = if succeed {
            env::var("BOLT_TEST_PASSWORD").unwrap()
        } else {
            String::from("invalid")
        };

        BoltConnectionManager::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok(),
            preferred_versions,
            HashMap::from_iter(vec![
                ("user_agent", "bolt-client/X.Y.Z"),
                ("scheme", "basic"),
                ("principal", &env::var("BOLT_TEST_USERNAME").unwrap()),
                ("credentials", &credentials),
            ]),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn basic_pool() {
        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], true).await;
            // Don't even test connection pool if server doesn't support this Bolt version
            if manager.connect().await.is_err() {
                println!(
                    "Skipping test: server doesn't support Bolt version {:#x}.",
                    bolt_version
                );
                continue;
            }
            let pool = Pool::builder().max_size(15).build(manager).await.unwrap();

            let mut tasks = Vec::with_capacity(50);
            for i in 1..=tasks.capacity() {
                let pool = pool.clone();
                tasks.push(async move {
                    let mut client = pool.get().await.unwrap();
                    let statement = format!("RETURN {} as num;", i);
                    let version = client.version();
                    let (response, records) = match version {
                        V1_0 | V2_0 => {
                            client.run(statement, None).await.unwrap();
                            client.pull_all().await.unwrap()
                        }
                        V3_0 => {
                            client
                                .run_with_metadata(statement, None, None)
                                .await
                                .unwrap();
                            client.pull_all().await.unwrap()
                        }
                        V4_0 | V4_1 => {
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
                });
            }
            join_all(tasks).await;
        }
    }

    #[tokio::test]
    async fn invalid_init_fails() {
        let invalid_manager = get_connection_manager([V4_1, V4_0, V3_0, V2_0], false).await;
        let pool = Pool::builder()
            .max_size(2)
            .build(invalid_manager)
            .await
            .unwrap();
        let conn = pool.dedicated_connection().await;
        assert!(matches!(conn, Err(Error::ClientInitFailed(_))));
    }
}
