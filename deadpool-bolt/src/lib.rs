#![warn(rust_2018_idioms)]

use std::{convert::TryFrom, io, net::SocketAddr};

use async_trait::async_trait;
use deadpool::managed::RecycleResult;
use thiserror::Error;
use tokio::{
    io::BufStream,
    net::{lookup_host, ToSocketAddrs},
};
use tokio_util::compat::*;

use bolt_client::{error::Error as ClientError, Metadata, Stream};
use bolt_proto::{error::Error as ProtocolError, message, Message};

pub use deadpool::{managed::PoolConfig, Runtime};

pub use bolt_client;
pub use bolt_proto;

pub struct Manager {
    addr: SocketAddr,
    domain: Option<String>,
    version_specifiers: [u32; 4],
    metadata: Metadata,
}

impl Manager {
    pub async fn new(
        addr: impl ToSocketAddrs,
        domain: Option<String>,
        version_specifiers: [u32; 4],
        metadata: Metadata,
    ) -> io::Result<Self> {
        Ok(Self {
            addr: lookup_host(addr)
                .await?
                .next()
                .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?,
            domain,
            version_specifiers,
            metadata,
        })
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid metadata: {0}")]
    InvalidMetadata(String),
    #[error("client initialization failed: received {0:?}")]
    ClientInitFailed(Message),
    #[error("unsupported client version: {0:#x}")]
    UnsupportedClientVersion(u32),
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
    #[error(transparent)]
    IoError(#[from] io::Error),
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
            &self.version_specifiers,
        )
        .await
        .map_err(ClientError::from)?;

        match client
            .hello(self.metadata.clone())
            .await
            .map_err(ClientError::from)?
        {
            Message::Success(_) => Ok(client),
            other => Err(Error::ClientInitFailed(other)),
        }
    }

    async fn recycle(&self, conn: &mut Client) -> RecycleResult<Error> {
        message::Success::try_from(
            conn.reset()
                .await
                .map_err(ClientError::from)
                .map_err::<Error, _>(Into::into)?,
        )
        .map_err(ProtocolError::from)
        .map_err::<Error, _>(Into::into)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{env, iter::FromIterator};

    use bolt_proto::{version::*, Value};
    use deadpool::managed::Manager as DeadpoolManager;
    use futures_util::{stream::FuturesUnordered, StreamExt};

    use super::*;

    async fn get_connection_manager(version_specifiers: [u32; 4], succeed: bool) -> Manager {
        let credentials = if succeed {
            env::var("BOLT_TEST_PASSWORD").unwrap()
        } else {
            String::from("invalid")
        };

        Manager::new(
            env::var("BOLT_TEST_ADDR").unwrap(),
            env::var("BOLT_TEST_DOMAIN").ok(),
            version_specifiers,
            Metadata::from_iter(vec![
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

        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1, V4_2, V4_3, V4] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], true).await;

            // Don't even test connection pool if server doesn't support this Bolt version
            if manager.create().await.is_err() {
                println!(
                    "Skipping test: server doesn't support Bolt version {:#x}.",
                    bolt_version
                );
                continue;
            }

            let pool = Pool::builder(manager).max_size(15).build().unwrap();

            (0..MAX_CONNS)
                .map(|i| {
                    let pool = pool.clone();
                    async move {
                        let mut client = pool.get().await.unwrap();
                        let statement = format!("RETURN {} as num;", i);
                        client.run(statement, None, None).await.unwrap();
                        let (records, response) = client
                            .pull(Some(Metadata::from_iter(vec![("n", 1)])))
                            .await
                            .unwrap();
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
        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1, V4_2, V4_3, V4] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], false).await;
            match manager.create().await {
                Ok(_) => panic!("initialization should have failed"),
                Err(Error::ClientError(ClientError::ConnectionError(
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
