#![warn(rust_2018_idioms)]

use std::{io, net::SocketAddr};

use async_trait::async_trait;
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
pub use mobc;

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
impl mobc::Manager for Manager {
    // TODO: Make a runtime-agnostic stream wrapper
    type Connection = Client<Compat<BufStream<Stream>>>;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
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

    async fn check(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        message::Success::try_from(conn.reset().await.map_err(Self::Error::from)?)
            .map_err(ProtocolError::from)
            .map_err(Self::Error::from)?;
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use bolt_proto::{version::*, Value};
    use futures_util::{stream::FuturesUnordered, StreamExt};
    use mobc::{Manager as MobcManager, Pool};

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
        const POOL_SIZE: u64 = 15;
        const MAX_CONNS: usize = 50;

        for &bolt_version in &[V1_0, V2_0, V3_0, V4_0, V4_1, V4_2, V4_3, V4] {
            let manager = get_connection_manager([bolt_version, 0, 0, 0], true).await;

            // Don't even test connection pool if server doesn't support this Bolt version
            match manager.connect().await {
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

            let pool = Pool::builder().max_open(POOL_SIZE).build(manager);

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
            match manager.connect().await {
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
