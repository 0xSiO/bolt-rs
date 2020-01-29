use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use bb8::ManageConnection;
use failure::{Error, Fail};

use async_trait::async_trait;
use bolt_client::message::Success;
use bolt_client::{Client, Message};

pub struct BoltConnectionManager {
    addr: SocketAddr,
    domain: Option<String>,
    client_name: String,
    auth_token: HashMap<String, String>,
}

impl BoltConnectionManager {
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
    #[fail(display = "Initialization of client failed: received {:?}", _0)]
    ClientInitFailed(Message),
}

#[async_trait]
impl ManageConnection for BoltConnectionManager {
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let mut client = match &self.domain {
            Some(domain) => Client::new_secure_tcp(domain, self.addr).await?,
            None => Client::new_tcp(self.addr).await?,
        };

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
    use std::env;
    use std::iter::FromIterator;

    use bb8::*;

    use super::*;

    fn get_connection_manager() -> BoltConnectionManager {
        let (addr, domain, username, password) = match env::var("BOLT_CLIENT_TEST_REMOTE") {
            Ok(domain) => {
                let login_info = env::var("BOLT_CLIENT_TEST_LOGIN").unwrap();
                let login_info: Vec<&str> = login_info.split(",").collect();
                (
                    domain.clone() + ":" + &env::var("BOLT_CLIENT_TEST_PORT").unwrap(),
                    Some(domain),
                    login_info[0].to_string(),
                    login_info[1].to_string(),
                )
            }
            Err(_) => (
                "127.0.0.1:7687".to_string(),
                None,
                "neo4j".to_string(),
                "test".to_string(),
            ),
        };

        BoltConnectionManager::new(
            addr,
            domain,
            "bolt-client/X.Y.Z".to_string(),
            HashMap::from_iter(vec![
                (String::from("scheme"), String::from("basic")),
                (String::from("principal"), username),
                (String::from("credentials"), password),
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
                assert!(Success::try_from(response).is_ok());
                assert_eq!(
                    i32::try_from(records[0].fields()[0].clone()).unwrap(),
                    i as i32
                );
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
