#![warn(rust_2018_idioms)]

//! [`bolt_client`] support for the [`deadpool`] connection pool.
//!
//! # Example
//! ```
//! use std::env;
//!
//! # use deadpool::managed::Manager as DeadpoolManager;
//! use deadpool_bolt::{
//!     bolt_client::Metadata,
//!     bolt_proto::{version::*, Value},
//!     Manager,
//!     Pool,
//! #   bolt_client::error::{ConnectionError, Error as ClientError},
//! #   bolt_proto::message::Success,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure a connection manager. We'll request Bolt v4.4 or v4.3.
//!     let manager = Manager::new(
//!         env::var("BOLT_TEST_ADDR")?,
//!         env::var("BOLT_TEST_DOMAIN").ok(),
//!         [V4_4, V4_3, 0, 0],
//!         Metadata::from_iter(vec![
//!             ("user_agent", "bolt-client/X.Y.Z"),
//!             ("scheme", "basic"),
//!             ("principal", &env::var("BOLT_TEST_USERNAME")?),
//!             ("credentials", &env::var("BOLT_TEST_PASSWORD")?),
//!         ]),
//!     ).await?;
//!
//! #   match manager.create().await {
//! #       Err(ClientError::ConnectionError(ConnectionError::HandshakeFailed(versions))) => {
//! #           println!("skipping test: {}", ConnectionError::HandshakeFailed(versions));
//! #           return Ok(());
//! #       }
//! #       Err(other) => panic!("{}", other),
//! #       _ => {}
//! #   }
//!     // Create a connection pool. This should be shared across your application.
//!     let pool = Pool::builder(manager).build()?;
//!
//!     // Fetch and use a connection from the pool
//!     let mut conn = pool.get().await?;
//!     let response = conn.run("RETURN 1 as num;", None, None).await?;
//! #   Success::try_from(response.clone()).unwrap();
//!     let pull_meta = Metadata::from_iter(vec![("n", 1)]);
//!     let (records, response) = conn.pull(Some(pull_meta)).await?;
//! #   Success::try_from(response).unwrap();
//!     assert_eq!(records[0].fields(), &[Value::from(1)]);
//!
//!     Ok(())
//! }

use std::{convert::Infallible, io, net::SocketAddr};

use async_trait::async_trait;
use deadpool::managed::RecycleResult;
use tokio::{
    io::BufStream,
    net::{lookup_host, ToSocketAddrs},
};
use tokio_util::compat::*;

use bolt_client::{
    error::{CommunicationError, ConnectionError, Error as ClientError},
    Client, Metadata, Stream,
};
use bolt_proto::{error::Error as ProtocolError, message, Message};

pub use bolt_client;
pub use bolt_client::bolt_proto;

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!(
    "bolt_client",
    Manager,
    deadpool::managed::Object<Manager>,
    ClientError,
    // We're not validating configuration before requesting connections
    Infallible
);

#[derive(Debug)]
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

#[async_trait]
impl deadpool::managed::Manager for Manager {
    // TODO: Make a runtime-agnostic stream wrapper
    type Type = Client<Compat<BufStream<Stream>>>;
    type Error = ClientError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let mut client = Client::new(
            BufStream::new(
                Stream::connect(self.addr, self.domain.as_ref())
                    .await
                    .map_err(ConnectionError::from)?,
            )
            .compat(),
            &self.version_specifiers,
        )
        .await?;

        match client.hello(self.metadata.clone()).await? {
            Message::Success(_) => Ok(client),
            other => Err(CommunicationError::from(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!("server responded with {:?}", other),
            ))
            .into()),
        }
    }

    async fn recycle(&self, conn: &mut Self::Type) -> RecycleResult<Self::Error> {
        message::Success::try_from(conn.reset().await.map_err(Self::Error::from)?)
            .map_err(ProtocolError::from)
            .map_err(Self::Error::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

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
        const POOL_SIZE: usize = 15;
        const MAX_CONNS: usize = 50;

        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1, V4_2, V4_3, V4_4, V4] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], true).await;

            // Don't even test connection pool if server doesn't support this Bolt version
            match manager.create().await {
                Err(ClientError::ConnectionError(ConnectionError::HandshakeFailed(versions))) => {
                    println!(
                        "skipping test: {}",
                        ConnectionError::HandshakeFailed(versions)
                    );
                    continue;
                }
                Err(other) => panic!("{}", other),
                _ => {}
            }

            let pool = Pool::builder(manager).max_size(POOL_SIZE).build().unwrap();

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
        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1, V4_2, V4_3, V4_4, V4] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], false).await;
            match manager.create().await {
                Ok(_) => panic!("initialization should have failed"),
                Err(ClientError::ConnectionError(ConnectionError::HandshakeFailed(versions))) => {
                    println!(
                        "skipping test: {}",
                        ConnectionError::HandshakeFailed(versions)
                    );
                    continue;
                }
                Err(ClientError::CommunicationError(comm_err)) => {
                    if let CommunicationError::IoError(io_err) = &*comm_err {
                        if io_err.kind() == io::ErrorKind::ConnectionAborted {
                            // Test passed. We only check the first compatible version since
                            // sending too many invalid credentials will cause us to get
                            // rate-limited.
                            return;
                        }
                    }
                    panic!("{}", comm_err);
                }
                Err(other) => panic!("{}", other),
            }
        }
    }
}
