//! An asynchronous client for Bolt-compatible servers.
//!
//! # Example
//! The below example demonstrates how to connect to a Neo4j server and send it Bolt messages.
//! ```
//! use std::collections::HashMap;
//! use std::convert::TryFrom;
//! use std::iter::FromIterator;
//!
//! use failure::Error;
//! use tokio::prelude::*;
//!
//! use bolt_client::*;
//! use bolt_client::message::*;
//! use bolt_client::value::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // Create a new connection to the server and perform a handshake to establish a
//!     // protocol version.
//!     let mut client = Client::new_tcp("127.0.0.1:7687").await?;
//!     
//!     // You can also create a TCP connection that is secured with TLS:
//!     // let mut client = Client::new_secure_tcp("mydomain.com", "mydomain.com:1234").await?;
//!
//!     // Send an INIT message with authorization details to the server to initialize
//!     // the session.
//!     let response_msg: Message = client.init(
//!         "my-client-name/1.0".to_string(),
//!         HashMap::from_iter(vec![
//!             ("scheme".to_string(), "basic".to_string()),
//!             ("principal".to_string(), "neo4j".to_string()),
//!             ("credentials".to_string(), "test".to_string()),
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
//!     // Note that integers are automatically packed into the smallest possible byte
//!     // representation.
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
pub use bolt_proto::message;
pub use bolt_proto::value;
pub use bolt_proto::{Message, Value};

#[doc(inline)]
pub use self::client::Client;

pub mod client;
mod stream;
