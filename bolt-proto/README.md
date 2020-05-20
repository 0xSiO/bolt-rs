This crate contains the traits and primitives used in the [Bolt](https://en.wikipedia.org/wiki/Bolt_%28network_protocol%29)
protocol. The `Message` and `Value` enums are of particular importance, and are the primary units of information sent and 
consumed by Bolt clients/servers.

The `Message` enum encapsulates all possible messages that can be sent between client and server.
```rust
pub enum Message {
    // V1-compatible message types
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

    // V3+-compatible message types
    Hello(Hello),
    Goodbye,
    RunWithMetadata(RunWithMetadata),
    Begin(Begin),
    Commit,
    Rollback,

    // V4+-compatible message types
    Discard(Discard),
    Pull(Pull),
}
```
See the [documentation](https://docs.rs/bolt-proto/*/bolt_proto/message/enum.Message.html) for more details.

The `Value` enum encapsulates all possible values that can be stored in data from each kind of `Message`. 
Structures like `List` and `Map` allow `Value`s to be nested with arbitrary complexity.
```rust
pub enum Value {
    // V1-compatible value types
    Boolean(Boolean),
    Integer(Integer),
    Float(Float),
    Bytes(ByteArray), // Added with Neo4j 3.2, no mention of it in the Bolt v1 docs!
    List(List),
    Map(Map),
    Null,
    String(String),
    Node(Node),
    Relationship(Relationship),
    Path(Path),
    UnboundRelationship(UnboundRelationship),

    // V2+-compatible value types
    Date(Date),                     // A date without a time zone, a.k.a. LocalDate
    Time(Time),                     // A time with a UTC offset, a.k.a. OffsetTime
    DateTimeOffset(DateTimeOffset), // A date-time with a UTC offset, a.k.a. OffsetDateTime
    DateTimeZoned(DateTimeZoned),   // A date-time with a time zone ID, a.k.a. ZonedDateTime
    LocalTime(LocalTime),           // A time without a time zone
    LocalDateTime(LocalDateTime),   // A date-time without a time zone
    Duration(Duration),
    Point2D(Point2D),
    Point3D(Point3D),
}
```
You should rarely ever have to construct variants directly (with the exception of `Value::Null`). Instead, you should
typically use `Value::from` on the type you wish to convert.
See the [documentation](https://docs.rs/bolt-proto/*/bolt_proto/value/enum.Value.html) for more details.

The `Serialize` and `Deserialize` traits provide interfaces for converting `Message` and `Value` types to and from 
streams of bytes, to be consumed by a compatible Bolt server.
