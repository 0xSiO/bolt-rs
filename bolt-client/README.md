This crate contains an asynchronous client for graph database servers that support the
[Bolt](https://7687.org/#bolt) protocol.

The central feature of this library is the
[`Client`](https://docs.rs/bolt-client/*/bolt_client/struct.Client.html) struct, which allows
sending Bolt messages to a compatible server.

An asynchronous TCP/TLS [`Stream`](https://docs.rs/bolt-client/*/bolt_client/enum.Stream.html)
wrapper is also available, if you're using the [tokio](https://tokio.rs/) runtime.

See the [API documentation](https://docs.rs/bolt-client) for more details and examples.
