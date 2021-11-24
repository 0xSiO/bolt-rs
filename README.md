# bolt-rs
[![CI](https://github.com/0xSiO/bolt-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/0xSiO/bolt-rs/actions/workflows/ci.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Overview

This project aims to provide a comprehensive set of libraries that allow for interaction with graph
database servers that support the [Bolt](https://7687.org/#bolt) protocol, namely,
[Neo4j](https://neo4j.com). This set of libraries allows interacting with servers supporting
versions 1 through 4.3 of the protocol, which includes Neo4j 3.1 through 4.3.

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

### mobc-bolt
[![crates.io](https://img.shields.io/crates/v/mobc-bolt.svg)](https://crates.io/crates/mobc-bolt)
[![Released API docs](https://docs.rs/mobc-bolt/badge.svg)](https://docs.rs/mobc-bolt)

A bolt-client manager for the [mobc](https://crates.io/crates/mobc) connection pool.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/0xSiO/bolt-rs.

## Donations

- XMR: `87abVq8rrb2QVDz9m63ZXeRB3efHxSWVjGisVWaeviuTU7aMNXEAi4wjoYpSzBn7vY7ikB62vRA8g8L75krFYMPs1ob5reh`

## License

These crates are available as open source under the terms of the
[MIT License](http://opensource.org/licenses/MIT), with portions of the documentation licensed under
the [Creative Commons Attribution-ShareAlike 4.0 International](https://creativecommons.org/licenses/by-sa/4.0/)
license.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
`bolt-rs` by you shall be licensed as MIT, without any additional terms or conditions.
