//! An asynchronous client for Bolt-compatible servers.
//!
//! # Example
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
//!     let mut client = Client::new("127.0.0.1".parse().unwrap(), 7687).await?;
//!     
//!     // Send an INIT message with authorization details to the server to initialize
//!     // the session.
//!     let response_msg: Message = client.init(
//!         "my-client-name/1.0".to_string(),
//!         HashMap::from_iter(vec![
//!             ("scheme".to_string(), Value::from("basic")),
//!             ("principal".to_string(), Value::from("neo4j")),
//!             ("credentials".to_string(), Value::from("test")),
//!         ])).await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!
//!     // Run a query on the server and retrieve the results
//!     let response_msg = client.run("RETURN 1 as num;".to_string(), None).await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!     let (response_msg, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!     // Note that integers are automatically packed into the smallest possible byte
//!     // representation.
//!     assert_eq!(records[0].fields(), &vec![Value::from(1 as i8)]);
//!
//!     // Run a more complex query with parameters
//!     client.run("CREATE (:Language {name: $name});".to_string(),
//!                Some(HashMap::from_iter(
//!                    vec![("name".to_string(), Value::from("Rust"))]
//!                ))).await?;
//!     client.pull_all().await?;
//!     client.run("MATCH (rust:Language) RETURN rust;".to_string(), None).await?;
//!     let (response_msg, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     assert!(Success::try_from(response_msg).is_ok());
//!     let node = Node::try_from(records[0].fields()[0].clone())?;
//!     // TODO: Check node properties    
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
