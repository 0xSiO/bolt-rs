This crate contains the traits and primitives used in the [Bolt v1](https://boltprotocol.org/v1/) protocol. 
The `Message` and `Value` enums are of particular importance, and are the primary units of information sent and 
consumed by Bolt clients/servers.

The `Message` enum encapsulates all possible messages that can be sent between client and server.
```rust
pub enum Message {
    Init(Init),
    Run(Run),
    DiscardAll,
    PullAll,
    AckFailure,
    Reset,
    Record(Record),
    Success(Success),
    Failure(Failure),
    Ignored,
}
```
See the [documentation](https://docs.rs/bolt-proto/*/bolt_proto/enum.Message.html) for more details.

The `Value` enum encapsulates all possible values that can be stored in data from each kind of `Message`. 
Structures like `List` and `Map` allow `Value`s to be nested with arbitrary complexity.
```rust
pub enum Value {
    Boolean(bool),
    Integer(Integer),
    Float(f64),
    List(List),
    Map(Map),
    Null,
    String(std::string::String),
    Node(Node),
    Relationship(Relationship),
    Path(Path),
    UnboundRelationship(UnboundRelationship),
}
```
You should rarely ever have to construct variants directly (with the exception of `Value::Null`). Instead, you should
typically use `Value::from` for the type you wish to convert.
See the [documentation](https://docs.rs/bolt-proto/*/bolt_proto/enum.Value.html) for more details.

The `Serialize` and `Deserialize` traits provide interfaces for converting `Message` and `Value` types to and from 
streams of bytes, to be consumed by a compatible Bolt server.
