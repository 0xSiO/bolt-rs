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
//! use bolt_client::*;
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
//!     let response: Message = client.hello(
//!         Metadata::from_iter(vec![
//!             ("user_agent", "my-client-name/1.0"),
//!             ("scheme", "basic"),
//!             ("principal", &env::var("BOLT_TEST_USERNAME")?),
//!             ("credentials", &env::var("BOLT_TEST_PASSWORD")?),
//!         ])).await?;
//!     assert!(Success::try_from(response).is_ok());
//!
//!     // Run a query on the server and retrieve the results
//!     let response = client.run_with_metadata(
//!         "RETURN 1 as num;", Default::default(), Default::default()
//!     ).await?;
//!     // Successful responses will include a SUCCESS message with related metadata
//!     // Consuming these messages is optional and will be skipped for the rest of the example
//!     assert!(Success::try_from(response).is_ok());
//!
//!     // Use PULL_ALL to retrieve results of the query, organized into RECORD messages
//!     let (response, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     # assert!(Success::try_from(response).is_ok());
//!
//!     // Integers are automatically packed into the smallest possible byte representation
//!     assert_eq!(records[0].fields(), &[Value::from(1 as i8)]);
//!     #
//!     # client.run_with_metadata("MATCH (n) DETACH DELETE n;", Default::default(), Default::default()).await?;
//!     # client.pull_all().await?;
//!
//!     // Run a more complex query with parameters
//!     let params = Params::from_iter(vec![("name", "Rust")]);
//!     client.run_with_metadata(
//!         "CREATE (:Client)-[:WRITTEN_IN]->(:Language {name: $name});",
//!         params, Default::default()).await?;
//!     client.pull_all().await?;
//!
//!     // Grab a node from the database and convert it to a native type
//!     client.run_with_metadata("MATCH (rust:Language) RETURN rust;",
//!         Default::default(), Default::default()).await?;
//!     let (response, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     # assert!(Success::try_from(response).is_ok());
//!     let node = Node::try_from(records[0].fields()[0].clone())?;
//!
//!     // Access properties from returned values
//!     assert_eq!(node.labels(), vec!["Language"]);
//!     assert_eq!(node.properties(),
//!                HashMap::from_iter(vec![("name", &Value::from("Rust"))]));
//!
//!     // End the connection with the server
//!     client.goodbye().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! For versions 1 and 2 of the protocol, the above example would have a few key differences:
//! ```
//! # use std::collections::HashMap;
//! # use std::convert::TryFrom;
//! # use std::env;
//! # use std::iter::FromIterator;
//! #
//! # use tokio::prelude::*;
//! #
//! # use bolt_client::*;
//! # use bolt_proto::{Message, Value};
//! # use bolt_proto::message::*;
//! # use bolt_proto::value::*;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let mut client = Client::new(env::var("BOLT_TEST_ADDR")?,
//! #                                  env::var("BOLT_TEST_DOMAIN").ok().as_deref()).await?;
//! // For the handshake we want to support versions 1 and 2 only, preferring version 2.
//! let handshake_result = client.handshake(&[2, 1, 0, 0]).await;
//!     # if let Err(bolt_client::error::Error::HandshakeFailed) = handshake_result {
//!     #     println!("Skipping test: client handshake failed");
//!     #     return Ok(());
//!     # }
//!     
//! // Instead of `hello`, we call `init`, and the user agent string is provided separately.
//! let response: Message = client.init(
//!     "my-client-name/1.0".to_string(),
//!     HashMap::from_iter(vec![
//!         ("scheme", "basic"),
//!         ("principal", &env::var("BOLT_TEST_USERNAME")?),
//!         ("credentials", &env::var("BOLT_TEST_PASSWORD")?),
//!     ])).await?;
//!     # assert!(Success::try_from(response).is_ok());
//!
//! // Instead of `run_with_metadata`, we call `run`, and there is no third parameter for metadata.
//! let response = client.run("RETURN 1 as num;", Default::default()).await?;
//!     # assert!(Success::try_from(response).is_ok());
//!     # let (response, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     # assert!(Success::try_from(response).is_ok());
//!     # assert_eq!(records[0].fields(), &[Value::from(1 as i8)]);
//!     #
//!     # client.run("MATCH (n) DETACH DELETE n;", Default::default()).await?;
//!     # client.pull_all().await?;
//!     #
//!     # client.run("CREATE (:Client)-[:WRITTEN_IN]->(:Language {name: $name});".to_string(),
//!     #            Params::from_iter(
//!     #                vec![("name".to_string(), Value::from("Rust"))]
//!     #            )).await?;
//!     # client.pull_all().await?;
//!     # client.run("MATCH (rust:Language) RETURN rust;", Default::default()).await?;
//!     # let (response, records): (Message, Vec<Record>) = client.pull_all().await?;
//!     # assert!(Success::try_from(response).is_ok());
//!     #
//!     # let node = Node::try_from(records[0].fields()[0].clone())?;
//!     # assert_eq!(node.labels(), &["Language".to_string()]);
//!     # assert_eq!(node.properties(),
//!     #            HashMap::from_iter(vec![("name", &Value::from("Rust"))]));
//!
//! // There is no call to `goodbye`
//!     # Ok(())
//! # }
//! ```
//! See the documentation of the `Client` struct for information on transaction management, error handling, and more.
#[doc(inline)]
pub use self::client::Client;

mod client;
mod define_value_map;
pub mod error;
mod stream;

define_value_map!(Metadata);
define_value_map!(Params);

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
