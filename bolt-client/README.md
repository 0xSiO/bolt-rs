This crate contains a runtime-agnostic asynchronous client for graph database servers that support
the [Bolt](https://7687.org/#bolt) protocol.

The central feature of this library is the
[`Client`](https://docs.rs/bolt-client/*/bolt_client/struct.Client.html) struct, which allows
sending Bolt messages to a compatible server. Clients can operate over any type that implements
[AsyncRead](https://docs.rs/futures-io/*/futures_io/trait.AsyncRead.html) and
[AsyncWrite](https://docs.rs/futures-io/*/futures_io/trait.AsyncRead.html).

If you want to connect to a Bolt-compatible server from your application, you probably want to use
a connection pool - see [bb8-bolt](https://crates.io/crates/bb8-bolt),
[deadpool-bolt](https://crates.io/crates/deadpool-bolt), or
[mobc-bolt](https://crates.io/crates/mobc-bolt).

If you'd rather manage your own connections, an asynchronous TCP/TLS
[`Stream`](https://docs.rs/bolt-client/*/bolt_client/enum.Stream.html) wrapper is also available,
if you're using the [tokio](https://tokio.rs/) runtime.

See the [API documentation](https://docs.rs/bolt-client) for more details and examples.
