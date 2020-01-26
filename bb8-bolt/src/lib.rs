use std::convert::TryFrom;
use std::net::IpAddr;

use bb8::ManageConnection;
use failure::Error;

use async_trait::async_trait;
use bolt_client::message::Success;
use bolt_client::Client;

pub struct BoltConnectionManager {
    host: IpAddr,
    port: u16,
}

impl BoltConnectionManager {
    pub fn new(host: IpAddr, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait]
impl ManageConnection for BoltConnectionManager {
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Client::new((self.host, self.port)).await
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
