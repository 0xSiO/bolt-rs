# bolt-rs
[![CI](https://github.com/lucis-fluxum/bolt-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/lucis-fluxum/bolt-rs/actions/workflows/ci.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Overview

This project aims to provide a comprehensive set of libraries that allow for interaction with graph
database servers that support the [Bolt](https://en.wikipedia.org/wiki/Bolt_%28network_protocol%29)
protocol, namely, [Neo4j](https://neo4j.com). This set of libraries allows interacting with servers
supporting versions 1 through 4.1 of the protocol, which includes Neo4j 3.1 through 4.2.

### bolt-proto
[![crates.io](https://img.shields.io/crates/v/bolt-proto.svg)](https://crates.io/crates/bolt-proto)
[![Released API docs](https://docs.rs/bolt-proto/badge.svg)](https://docs.rs/bolt-proto)

Contains the traits and primitives used in the protocol. The `Message` and `Value` enums are of
particular importance, and are the primary units of information sent and consumed by Bolt
clients/servers.

### bolt-client
[![crates.io](https://img.shields.io/crates/v/bolt-client.svg)](https://crates.io/crates/bolt-client)
[![Released API docs](https://docs.rs/bolt-client/badge.svg)](https://docs.rs/bolt-client)

Contains a runtime-agnostic asynchronous client for Bolt-compatible servers, as well as an optional
tokio-based `Stream` type that supports both insecure and secure TCP streams backed by
[rustls](https://docs.rs/rustls).

### bb8-bolt
[![crates.io](https://img.shields.io/crates/v/bb8-bolt.svg)](https://crates.io/crates/bb8-bolt)
[![Released API docs](https://docs.rs/bb8-bolt/badge.svg)](https://docs.rs/bb8-bolt)

A bolt-client adapter crate for the [bb8](https://crates.io/crates/bb8) connection pool.

### deadpool-bolt
[![crates.io](https://img.shields.io/crates/v/deadpool-bolt.svg)](https://crates.io/crates/deadpool-bolt)
[![Released API docs](https://docs.rs/deadpool-bolt/badge.svg)](https://docs.rs/deadpool-bolt)

A bolt-client manager for the [deadpool](https://crates.io/crates/deadpool) connection pool.

### bolt-proto-derive
[![crates.io](https://img.shields.io/crates/v/bolt-proto-derive.svg)](https://crates.io/crates/bolt-proto-derive)
[![Released API docs](https://docs.rs/bolt-proto-derive/badge.svg)](https://docs.rs/bolt-proto-derive)

Procedural macros used in bolt-proto to derive serialization-related traits.

### bolt-client-macros
[![crates.io](https://img.shields.io/crates/v/bolt-client-macros.svg)](https://crates.io/crates/bolt-client-macros)
[![Released API docs](https://docs.rs/bolt-client-macros/badge.svg)](https://docs.rs/bolt-client-macros)

Procedural macros used in bolt-client for client version requirements and smarter tests.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/lucis-fluxum/bolt-rs.

## License

These crates are available as open source under the terms of the
[MIT License](http://opensource.org/licenses/MIT), with portions of the documentation licensed under
the [Creative Commons Attribution-ShareAlike 3.0 License](https://creativecommons.org/licenses/by-sa/3.0/).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`bolt-rs` by you shall be licensed as MIT, without any additional terms or conditions.
