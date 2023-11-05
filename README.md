# bolt-rs
[![CI](https://github.com/0xSiO/bolt-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/0xSiO/bolt-rs/actions/workflows/ci.yml)

## Overview

This project aims to provide a comprehensive set of libraries that allow for interaction with graph
database servers that support the [Bolt](https://neo4j.com/docs/bolt/current) protocol, namely,
[Neo4j](https://neo4j.com). This set of libraries allows interacting with servers supporting
versions 1 through 4.4 of the protocol, which includes Neo4j 3.1 through 4.4.

### bolt-proto
[![crates.io](https://img.shields.io/crates/v/bolt-proto.svg)](https://crates.io/crates/bolt-proto)
[![Released API docs](https://docs.rs/bolt-proto/badge.svg)](https://docs.rs/bolt-proto)

Contains the primitives used in the protocol. The `Message` and `Value` enums are of particular
importance, and are the primary units of information sent and consumed by Bolt clients/servers.

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

- Contributions to this project must be submitted under the [project's license](./LICENSE).
- Contributors to this project must attest to the [Developer Certificate of Origin](https://developercertificate.org/) by including a `Signed-off-by` statement in all commit messages.
- All commits must have a valid digital signature.
