# bolt-rs
[![Build Status](https://travis-ci.org/lucis-fluxum/bolt-rs.svg?branch=master)](https://travis-ci.org/lucis-fluxum/bolt-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Overview

This project aims to provide a comprehensive set of libraries that allow for interaction with graph database servers that
support the [Bolt](https://en.wikipedia.org/wiki/Bolt_%28network_protocol%29) protocol, namely, [Neo4j](https://neo4j.com).
This set of libraries allows interacting with servers supporting versions 1 through 3 of the protocol, which includes 
Neo4j 4.0. Development to support the newer versions of the protocol is ongoing. The project roadmap is shown below:
- [x] Bolt v1 protocol
    - [x] Messaging and serialization
- [x] Bolt v2 protocol
    - [x] New data types (dates, times, durations, points)
- [x] Bolt v3 protocol
    - [x] New message types (Hello, Goodbye, RunWithMetadata, Begin, Commit, Rollback)
- [x] Bolt v4 protocol
    - [x] New message types (Pull, Discard)
- [ ] Client and connection pool adaptor
    - [x] v1-v3 client behavior
    - [ ] v4 client behavior
    - [ ] Implement transaction handling/retries (or leave it to a higher-level library)
    - [ ] Benchmarks?
- [ ] Address TODOs scattered throughout codebase

### bolt-client
[![crates.io](https://img.shields.io/crates/v/bolt-client.svg)](https://crates.io/crates/bolt-client)
[![Released API docs](https://docs.rs/bolt-client/badge.svg)](https://docs.rs/bolt-client)

Contains an asynchronous client for Bolt-compatible servers, using a TCP stream optionally secured using
TLS.

### bolt-client-macros
[![crates.io](https://img.shields.io/crates/v/bolt-client-macros.svg)](https://crates.io/crates/bolt-client-macros)
[![Released API docs](https://docs.rs/bolt-client-macros/badge.svg)](https://docs.rs/bolt-client-macros)

Procedural macros used in bolt-client for client version requirements and smarter tests.

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

These crates are available as open source under the terms of the [MIT License](http://opensource.org/licenses/MIT), with
portions of the documentation licensed under the 
[Creative Commons Attribution-ShareAlike 3.0 License](https://creativecommons.org/licenses/by-sa/3.0/).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `bolt-rs` by you shall
be licensed as MIT, without any additional terms or conditions.
