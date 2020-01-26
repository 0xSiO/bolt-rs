use std::future::Future;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use bb8::ManageConnection;
use failure::{Compat, Error, Fail, Fallible};

use async_trait::async_trait;
use bolt_client::Client;

pub struct BoltConnectionManager {
    host: IpAddr,
    port: usize,
}

#[async_trait]
impl ManageConnection for BoltConnectionManager {
    type Connection = Client;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        unimplemented!()
    }

    async fn is_valid(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        unimplemented!()
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        unimplemented!()
    }
}
