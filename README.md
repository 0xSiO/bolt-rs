# bolt-rs
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Overview

This project provides a comprehensive set of libraries that allow for interaction with graph database servers that
support the [Bolt v1](https://boltprotocol.org/v1/) protocol, namely, [Neo4j](https://neo4j.com). Since there is
currently no good documentation for Bolt v2, v3, or v4, this set of libraries allows interacting with Neo4j versions up
to 3.5.14, which is the last version supporting Bolt v1.

### bolt-client
[![crates.io](https://img.shields.io/crates/v/bolt-client.svg)](https://crates.io/crates/bolt-client)
[![Released API docs](https://docs.rs/bolt-client/badge.svg)](https://docs.rs/bolt-client)

Contains an asynchronous client for Bolt-compatible servers, using a [tokio](https://crates.io/crates/tokio) 
[`BufStream`](https://docs.rs/tokio/0.2.10/tokio/io/struct.BufStream.html) wrapping a 
[`TcpStream`](https://docs.rs/tokio/0.2.10/tokio/net/struct.TcpStream.html), optionally secured using TLS.

### bolt-proto
[![crates.io](https://img.shields.io/crates/v/bolt-proto.svg)](https://crates.io/crates/bolt-proto)
[![Released API docs](https://docs.rs/bolt-proto/badge.svg)](https://docs.rs/bolt-proto)

Contains the traits and primitives used in the protocol. The `Message` and `Value` enums are of particular importance,
and are the primary units of information sent and consumed by Bolt clients/servers.

### bolt-proto-derive
[![crates.io](https://img.shields.io/crates/v/bolt-proto-derive.svg)](https://crates.io/crates/bolt-proto-derive)
[![Released API docs](https://docs.rs/bolt-proto-derive/badge.svg)](https://docs.rs/bolt-proto-derive)

Ugly procedural macros used in bolt-proto to derive serialization-related traits.

### bb8-bolt
[![crates.io](https://img.shields.io/crates/v/bb8-bolt.svg)](https://crates.io/crates/bb8-bolt)
[![Released API docs](https://docs.rs/bb8-bolt/badge.svg)](https://docs.rs/bb8-bolt)

A bolt-client adapter crate for the [bb8](https://crates.io/crates/bb8) connection pool.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/lucis-fluxum/bolt-rs.

## License

This crate is available as open source under the terms of the [MIT License](http://opensource.org/licenses/MIT), with
portions of the documentation licensed under the 
[Creative Commons Attribution-ShareAlike 3.0 License](https://creativecommons.org/licenses/by-sa/3.0/).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `bolt-rs` by you shall
be licensed as MIT, without any additional terms or conditions.
