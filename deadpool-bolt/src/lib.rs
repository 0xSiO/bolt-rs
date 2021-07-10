#![warn(rust_2018_idioms)]

use std::{collections::HashMap, convert::TryFrom, net::SocketAddr};

use async_trait::async_trait;
use deadpool::managed::RecycleResult;
use thiserror::Error;
use tokio::{
    io::BufStream,
    net::{lookup_host, ToSocketAddrs},
};
use tokio_util::compat::*;

use bolt_client::*;
use bolt_proto::{version::*, *};

pub use deadpool::{managed::PoolConfig, Runtime};

pub use bolt_client;
pub use bolt_proto;

pub struct Manager {
    addr: SocketAddr,
    domain: Option<String>,
    preferred_versions: [u32; 4],
    metadata: HashMap<String, Value>,
}

impl Manager {
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
    ConversionError(#[from] bolt_proto::error::ConversionError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

type Client = bolt_client::Client<Compat<BufStream<Stream>>>;
pub type Connection = deadpool::managed::Object<Manager>;
pub type Pool = deadpool::managed::Pool<Manager>;
pub type PoolError = deadpool::managed::PoolError<Error>;

#[async_trait]
impl deadpool::managed::Manager for Manager {
    type Type = Client;
    type Error = Error;

    async fn create(&self) -> Result<Client, Error> {
        let mut client = Client::new(
            BufStream::new(Stream::connect(self.addr, self.domain.as_ref()).await?).compat(),
            &self.preferred_versions,
        )
        .await
        .map_err(bolt_client::error::Error::from)?;

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

    async fn recycle(&self, conn: &mut Client) -> RecycleResult<Error> {
        let response = conn
            .run("RETURN 1;".to_string(), None)
            .await
            .map_err(Error::from)?;
        message::Success::try_from(response).map_err(Error::from)?;
        let (response, _records) = conn.pull_all().await.map_err(Error::from)?;
        message::Success::try_from(response).map_err(Error::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::iter::FromIterator;

    use deadpool::managed::Manager as DeadpoolManager;
    use futures_util::{stream::FuturesUnordered, StreamExt};

    use super::*;

    async fn get_connection_manager(preferred_versions: [u32; 4], succeed: bool) -> Manager {
        let credentials = if succeed {
            env::var("BOLT_TEST_PASSWORD").unwrap()
        } else {
            String::from("invalid")
        };

        Manager::new(
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn basic_pool() {
        const MAX_CONNS: usize = 50;

        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], true).await;

            // Don't even test connection pool if server doesn't support this Bolt version
            if manager.create().await.is_err() {
                println!(
                    "Skipping test: server doesn't support Bolt version {:#x}.",
                    bolt_version
                );
                continue;
            }

            let pool = Pool::new(manager, 15);

            (0..MAX_CONNS)
                .map(|i| {
                    let pool = pool.clone();
                    async move {
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
                    }
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;
        }
    }

    #[tokio::test]
    async fn invalid_init_fails() {
        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], false).await;
            match manager.create().await {
                Ok(_) => panic!("initialization should have failed"),
                Err(Error::ClientError(bolt_client::error::Error::ConnectionError(
                    bolt_client::error::ConnectionError::HandshakeFailed(_),
                ))) => {
                    println!(
                        "Skipping test: server doesn't support Bolt version {:#x}.",
                        bolt_version
                    );
                    continue;
                }
                Err(Error::ClientInitFailed(_)) => {
                    // Test passed. We only check the first compatible version since sending too
                    // many invalid credentials will cause us to get rate-limited.
                    return;
                }
                Err(other) => panic!("{}", other),
            }
        }
    }
}
