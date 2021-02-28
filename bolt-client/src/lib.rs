#![cfg_attr(docsrs, feature(doc_cfg))]

//! An asynchronous client for Bolt-compatible servers.
//!
//! # Example
//! The below example demonstrates how to communicate with a Neo4j server using Bolt
//! protocol version 4.
//! ```
//! use std::collections::HashMap;
//! use std::convert::TryFrom;
//! use std::env;
//! use std::iter::FromIterator;
//!
//! use tokio::io::BufStream;
//! use tokio_util::compat::*;
//!
//! use bolt_client::*;
//! use bolt_proto::{message::*, value::*, version::*, Message, Value};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Let's say you have a type that implements AsyncRead + AsyncWrite. Here's one
//!     // provided by the `tokio-stream` feature of this library. In this example, all
//!     // connection/authentication details are stored in environment variables.
//!     let stream = Stream::connect(env::var("BOLT_TEST_ADDR")?,
//!                                  env::var("BOLT_TEST_DOMAIN").ok()).await?;
//!     let stream = BufStream::new(stream).compat();
//!
//!     // Create a new connection to the server and perform a handshake to establish a
//!     // protocol version. This example demonstrates usage of the v4.1 or v4 protocol.
//!     let mut result = Client::new(stream, &[V4_1, V4_0, 0, 0]).await;
//! #   skip_if_handshake_failed!(result, Ok(()));
//!     let mut client = result.unwrap();
//!     
//!     // Send a HELLO message with authorization details to the server to initialize
//!     // the session.
//!     let response: Message = client.hello(
//!         Some(Metadata::from_iter(vec![
//!             ("user_agent", "my-client-name/1.0"),
//!             ("scheme", "basic"),
//!             ("principal", &env::var("BOLT_TEST_USERNAME")?),
//!             ("credentials", &env::var("BOLT_TEST_PASSWORD")?),
//!         ]))).await?;
//!     assert!(Success::try_from(response).is_ok());
//!
//!     // Run a query on the server
//!     let response = client.run_with_metadata("RETURN 1 as num;", None, None).await?;
//!
//!     // Successful responses will include a SUCCESS message with related metadata
//!     // Consuming these messages is optional and will be skipped for the rest of the example
//!     assert!(Success::try_from(response).is_ok());
//!
//!     // Use PULL to retrieve results of the query, organized into RECORD messages
//!     // We get a (Message, Vec<Record>) returned from a PULL
//!     let pull_meta = Metadata::from_iter(vec![("n", 1)]);
//!     let (response, records) = client.pull(Some(pull_meta.clone())).await?;
//! #   assert!(Success::try_from(response).is_ok());
//!
//!     assert_eq!(records[0].fields(), &[Value::from(1)]);
//! #    
//! #   client.run_with_metadata("MATCH (n) DETACH DELETE n;", None, None).await?;
//! #   client.pull(Some(pull_meta.clone())).await?;
//!
//!     // Run a more complex query with parameters
//!     let params = Params::from_iter(vec![("name", "Rust")]);
//!     client.run_with_metadata(
//!         "CREATE (:Client)-[:WRITTEN_IN]->(:Language {name: $name});",
//!         Some(params), None).await?;
//!     client.pull(Some(pull_meta.clone())).await?;
//!
//!     // Grab a node from the database and convert it to a native type
//!     client.run_with_metadata("MATCH (rust:Language) RETURN rust;", None, None).await?;
//!     let (response, records) = client.pull(Some(pull_meta.clone())).await?;
//! #   assert!(Success::try_from(response).is_ok());
//!     let node = Node::try_from(records[0].fields()[0].clone())?;
//!
//!     // Access properties from returned values
//!     assert_eq!(node.labels(), &[String::from("Language")]);
//!     assert_eq!(node.properties(),
//!                &HashMap::from_iter(vec![(String::from("name"), Value::from("Rust"))]));
//!
//!     // End the connection with the server
//!     client.goodbye().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! For version 3 of the protocol, the above example would simply use [`Client::pull_all`]
//! instead of [`Client::pull`]. In version 4, note that we must pass metadata to `PULL`
//! to indicate how many records we wish to consume, but in version 3 this metadata is not
//! required (i.e. all records are consumed).
//! ```
//! # use std::collections::HashMap;
//! # use std::convert::TryFrom;
//! # use std::env;
//! # use std::iter::FromIterator;
//! #
//! # use tokio::io::BufStream;
//! # use tokio_util::compat::*;
//! #
//! # use bolt_client::*;
//! # use bolt_proto::{message::*, value::*, version::*, Message, Value};
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let stream = Stream::connect(env::var("BOLT_TEST_ADDR")?,
//! #                                  env::var("BOLT_TEST_DOMAIN").ok()).await?;
//! #     let stream = BufStream::new(stream).compat();
//! // Now we only want Bolt v3
//! let mut result = Client::new(stream, &[V3_0, 0, 0, 0]).await;
//! #     skip_if_handshake_failed!(result, Ok(()));
//! #     let mut client = result.unwrap();
//! #
//! #     let response: Message = client.hello(
//! #         Some(Metadata::from_iter(vec![
//! #             ("user_agent", "my-client-name/1.0"),
//! #             ("scheme", "basic"),
//! #             ("principal", &env::var("BOLT_TEST_USERNAME")?),
//! #             ("credentials", &env::var("BOLT_TEST_PASSWORD")?),
//! #         ]))).await?;
//! #     assert!(Success::try_from(response).is_ok());
//! #
//! #     let response = client.run_with_metadata("RETURN 1 as num;", None, None).await?;
//! #     assert!(Success::try_from(response).is_ok());
//!
//! // PULL_ALL instead of PULL
//! let (response, records) = client.pull_all().await?;
//! #     assert!(Success::try_from(response).is_ok());
//! #
//! #     assert_eq!(records[0].fields(), &[Value::from(1 as i8)]);
//! #     client.run_with_metadata("MATCH (n {test: 'doctest-v3'}) DETACH DELETE n;", None, None).await?;
//! #     client.pull_all().await?;
//! #
//! #     let params = Params::from_iter(vec![("name", "Rust")]);
//! #     client.run_with_metadata(
//! #         "CREATE (:Client {test: 'doctest-v3'})-[:WRITTEN_IN]->(:Language {name: $name, test: 'doctest-v3'});",
//! #         Some(params), None).await?;
//! #     client.pull_all().await?;
//! #
//! #     client.run_with_metadata("MATCH (rust:Language {test: 'doctest-v3'}) RETURN rust;", None, None).await?;
//! #     let (response, records): (Message, Vec<Record>) = client.pull_all().await?;
//! #     assert!(Success::try_from(response).is_ok());
//! #     let node = Node::try_from(records[0].fields()[0].clone())?;
//! #     assert_eq!(node.labels(), &[String::from("Language")]);
//! #     assert_eq!(node.properties(),
//! #                &HashMap::from_iter(vec![(String::from("name"), Value::from("Rust")),
//! #                                         (String::from("test"), Value::from("doctest-v3"))]));
//! #     client.goodbye().await?;
//! #     Ok(())
//! # }
//! ```
//!
//! For versions 1 and 2 of the protocol, the changes are more involved:
//! ```
//! # use std::collections::HashMap;
//! # use std::convert::TryFrom;
//! # use std::env;
//! # use std::iter::FromIterator;
//! #
//! # use tokio::io::BufStream;
//! # use tokio_util::compat::*;
//! #
//! # use bolt_client::*;
//! # use bolt_proto::{message::*, value::*, version::*, Message, Value};
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     let stream = Stream::connect(env::var("BOLT_TEST_ADDR")?,
//! #                                  env::var("BOLT_TEST_DOMAIN").ok()).await?;
//! #     let stream = BufStream::new(stream).compat();
//! // For the handshake we want to support versions 1 and 2 only, preferring version 2.
//! let mut result = Client::new(stream, &[V2_0, V1_0, 0, 0]).await;
//! #     skip_if_handshake_failed!(result, Ok(()));
//! #     let mut client = result.unwrap();
//!     
//! // Instead of `hello`, we call `init`, and the user agent string is provided separately.
//! let response: Message = client.init(
//!     "my-client-name/1.0",
//!     Metadata::from_iter(vec![
//!         ("scheme", "basic"),
//!         ("principal", &env::var("BOLT_TEST_USERNAME")?),
//!         ("credentials", &env::var("BOLT_TEST_PASSWORD")?),
//!     ])).await?;
//! #     assert!(Success::try_from(response).is_ok());
//!
//! // Instead of `run_with_metadata`, we call `run`, and there is no third parameter for metadata.
//! let response = client.run("RETURN 1 as num;", None).await?;
//! #     assert!(Success::try_from(response).is_ok());
//!
//! // We also use Client::pull_all here.
//! let (response, records) = client.pull_all().await?;
//! #     assert!(Success::try_from(response).is_ok());
//! #     assert_eq!(records[0].fields(), &[Value::from(1 as i8)]);
//! #    
//! #     client.run("MATCH (n {test: 'doctest-v2-v1'}) DETACH DELETE n;", None).await?;
//! #     client.pull_all().await?;
//! #    
//! #     client.run("CREATE (:Client {test: 'doctest-v2-v1'})-[:WRITTEN_IN]->(:Language {name: $name, test: 'doctest-v2-v1'});",
//! #                Some(Params::from_iter(
//! #                    vec![("name".to_string(), Value::from("Rust"))]
//! #                ))).await?;
//! #     client.pull_all().await?;
//! #     client.run("MATCH (rust:Language {test: 'doctest-v2-v1'}) RETURN rust;", None).await?;
//! #     let (response, records): (Message, Vec<Record>) = client.pull_all().await?;
//! #     assert!(Success::try_from(response).is_ok());
//! #    
//! #     let node = Node::try_from(records[0].fields()[0].clone())?;
//! #     assert_eq!(node.labels(), &["Language".to_string()]);
//! #     assert_eq!(node.properties(),
//! #                &HashMap::from_iter(vec![(String::from("name"), Value::from("Rust")),
//! #                                         (String::from("test"), Value::from("doctest-v2-v1"))]));
//!
//! // There is no call to `goodbye`
//! #     Ok(())
//! # }
//! ```
//! See the documentation of the [`Client`] struct for information on transaction
//! management, error handling, and more.
#[doc(inline)]
pub use self::client::Client;

mod client;
mod define_value_map;
pub mod error;

pub use bolt_proto;

#[cfg(feature = "tokio-stream")]
mod stream;

#[cfg(feature = "tokio-stream")]
pub use stream::Stream;

define_value_map!(Metadata);
define_value_map!(Params);

#[doc(hidden)]
#[macro_export]
macro_rules! skip_if_handshake_failed {
    ($var:expr) => {
        if let ::std::result::Result::Err($crate::error::Error::HandshakeFailed(versions)) = $var {
            println!(
                "Skipping test: {}",
                $crate::error::Error::HandshakeFailed(versions)
            );
            return;
        }
    };
    ($var:expr, $ret:expr) => {
        if let ::std::result::Result::Err($crate::error::Error::HandshakeFailed(versions)) = $var {
            println!(
                "Skipping test: {}",
                $crate::error::Error::HandshakeFailed(versions)
            );
            return $ret;
        }
    };
}
