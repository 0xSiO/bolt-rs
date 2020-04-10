//! An asynchronous client for Bolt-compatible servers.
//!
//! # Example
//! The below example demonstrates how to communicate with a Neo4j server using Bolt protocol version 3.
//! ```
//! use std::collections::HashMap;
//! use std::convert::TryFrom;
//! use std::env;
//! use std::iter::FromIterator;
//!
//! use tokio::prelude::*;
//!
//! use bolt_client::Client;
//! use bolt_proto::{Message, Value};
//! use bolt_proto::message::*;
//! use bolt_proto::value::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new connection to the server and perform a handshake to establish a
//!     // protocol version. In this example, all connection/authentication details are
//!     // stored in environment variables. A domain is optional - including it will
//!     // create a client that uses a TLS-secured connection.
//!     let mut client = Client::new(env::var("BOLT_TEST_ADDR")?,
//!                                  env::var("BOLT_TEST_DOMAIN").ok().as_deref()).await?;
//!     // This example demonstrates usage of the v3 protocol
//!     let handshake_result = client.handshake(&[3, 0, 0, 0]).await;
//!     # if let Err(bolt_client::error::Error::HandshakeFailed) = handshake_result {
//!     #     println!("Skipping test: client handshake failed");
//!     #     return Ok(());
//!     # }
//!     
//!     // Send a HELLO message with authorization details to the server to initialize
//!     // the session.
//!     let response_msg: Message = client.hello(
//!         HashMap::from_iter(vec![
//!             ("user_agent".to_string(), "my-client-name/1.0".to_string()),
//!             ("scheme".to_string(), "basic".to_string()),
//!             ("principal".to_string(), env::var("BOLT_TEST_USERNAME")?),
//!             ("credentials".to_string(), env::var("BOLT_TEST_PASSWORD")?),
//!         ])).await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Run a query on the server and retrieve the results
//!     let response_msg = client.run_with_metadata("RETURN 1 as num;".to_string(), None, None).await?;
//!     // Successful RUN messages will return a SUCCESS message with related metadata
//!     // Consuming these messages is optional and will be skipped for the rest of the example
//!     assert!(Success::try_from(response_msg).is_ok());
//!     // Use PULL_ALL to retrieve results of the query
//!     let (response_msg, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Integers are automatically packed into the smallest possible byte representation
//!     assert_eq!(records[0].fields(), &[Value::from(1 as i8)]);
//!
//!     // Clear the database
//!     client.run_with_metadata("MATCH (n) DETACH DELETE n;".to_string(), None, None).await?;
//!     client.pull_all().await?;
//!
//!     // Run a more complex query with parameters
//!     client.run_with_metadata("CREATE (:Client)-[:WRITTEN_IN]->(:Language {name: $name});".to_string(),
//!                              Some(HashMap::from_iter(
//!                                  vec![("name".to_string(), Value::from("Rust"))]
//!                              )),
//!                              None).await?;
//!     client.pull_all().await?;
//!     client.run_with_metadata("MATCH (rust:Language) RETURN rust;".to_string(), None, None).await?;
//!     let (response_msg, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Access properties from returned values
//!     let node = Node::try_from(records[0].fields()[0].clone())?;
//!     assert_eq!(node.labels(), &["Language".to_string()]);
//!     assert_eq!(node.properties(),
//!                &HashMap::from_iter(vec![("name".to_string(), Value::from("Rust"))]));
//!
//!     Ok(())
//! }
//! ```
//!
//! The below example demonstrates how to communicate with a Neo4j server using Bolt protocol version 1 or 2.
//! ```
//! use std::collections::HashMap;
//! use std::convert::TryFrom;
//! use std::env;
//! use std::iter::FromIterator;
//!
//! use tokio::prelude::*;
//!
//! use bolt_client::Client;
//! use bolt_proto::{Message, Value};
//! use bolt_proto::message::*;
//! use bolt_proto::value::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new connection to the server and perform a handshake to establish a
//!     // protocol version. In this example, all connection/authentication details are
//!     // stored in environment variables. A domain is optional - including it will
//!     // create a client that uses a TLS-secured connection.
//!     let mut client = Client::new(env::var("BOLT_TEST_ADDR")?,
//!                                  env::var("BOLT_TEST_DOMAIN").ok().as_deref()).await?;
//!     // This example demonstrates usage of the v1 or v2 protocol
//!     let handshake_result = client.handshake(&[2, 1, 0, 0]).await;
//!     # if let Err(bolt_client::error::Error::HandshakeFailed) = handshake_result {
//!     #     println!("Skipping test: client handshake failed");
//!     #     return Ok(());
//!     # }
//!     
//!     // Send an INIT message with authorization details to the server to initialize
//!     // the session.
//!     let response_msg: Message = client.init(
//!         "my-client-name/1.0".to_string(),
//!         HashMap::from_iter(vec![
//!             ("scheme".to_string(), "basic".to_string()),
//!             ("principal".to_string(), env::var("BOLT_TEST_USERNAME")?),
//!             ("credentials".to_string(), env::var("BOLT_TEST_PASSWORD")?),
//!         ])).await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Run a query on the server and retrieve the results
//!     let response_msg = client.run("RETURN 1 as num;".to_string(), None).await?;
//!     // Successful RUN messages will return a SUCCESS message with related metadata
//!     // Consuming these messages is optional and will be skipped for the rest of the example
//!     assert!(Success::try_from(response_msg).is_ok());
//!     // Use PULL_ALL to retrieve results of the query
//!     let (response_msg, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Integers are automatically packed into the smallest possible byte representation
//!     assert_eq!(records[0].fields(), &[Value::from(1 as i8)]);
//!
//!     // Clear the database
//!     client.run("MATCH (n) DETACH DELETE n;".to_string(), None).await?;
//!     client.pull_all().await?;
//!
//!     // Run a more complex query with parameters
//!     client.run("CREATE (:Client)-[:WRITTEN_IN]->(:Language {name: $name});".to_string(),
//!                Some(HashMap::from_iter(
//!                    vec![("name".to_string(), Value::from("Rust"))]
//!                ))).await?;
//!     client.pull_all().await?;
//!     client.run("MATCH (rust:Language) RETURN rust;".to_string(), None).await?;
//!     let (response_msg, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Access properties from returned values
//!     let node = Node::try_from(records[0].fields()[0].clone())?;
//!     assert_eq!(node.labels(), &["Language".to_string()]);
//!     assert_eq!(node.properties(),
//!                &HashMap::from_iter(vec![("name".to_string(), Value::from("Rust"))]));
//!
//!     Ok(())
//! }
//! ```
#[doc(inline)]
pub use self::client::Client;

pub mod client;
pub mod error;
mod stream;

#[doc(hidden)]
#[macro_export]
macro_rules! skip_if_handshake_failed {
    ($var:expr) => {
        if let ::std::result::Result::Err(crate::error::Error::HandshakeFailed) = $var {
            println!("Skipping test: client handshake failed");
            return;
        }
    };
    ($var:expr, $ret:expr) => {
        if let ::std::result::Result::Err(crate::error::Error::HandshakeFailed) = $var {
            println!("Skipping test: client handshake failed");
            return $ret;
        }
    };
}
