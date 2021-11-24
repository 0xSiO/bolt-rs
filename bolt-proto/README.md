This crate contains the traits and primitives used in the [Bolt](https://7687.org/#bolt) protocol.
The `Message` and `Value` enums are of particular importance, and are the primary units of
information sent and consumed by Bolt clients/servers.

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

    // V4.3+-compatible message types
    Route(Route),
}
```
See the [documentation](https://docs.rs/bolt-proto/*/bolt_proto/message/enum.Message.html) for more
details.

The `Value` enum encapsulates all possible values that can be sent in each kind of `Message`.
Structures like `List` and `Map` allow `Value`s to be nested with arbitrary complexity.
```rust
pub enum Value {
    // V1-compatible value types
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Null,
    String(String),
    Node(Node),
    Relationship(Relationship),
    Path(Path),
    UnboundRelationship(UnboundRelationship),

    // V2+-compatible value types
    Date(NaiveDate),                       // A date without a time zone, i.e. LocalDate
    Time(NaiveTime, FixedOffset),          // A time with UTC offset, i.e. OffsetTime
    DateTimeOffset(DateTime<FixedOffset>), // A date-time with UTC offset, i.e. OffsetDateTime
    DateTimeZoned(DateTime<Tz>),           // A date-time with time zone ID, i.e. ZonedDateTime
    LocalTime(NaiveTime),                  // A time without time zone
    LocalDateTime(NaiveDateTime),          // A date-time without time zone
    Duration(Duration),
    Point2D(Point2D),
    Point3D(Point3D),
}
```
You should rarely ever have to construct variants directly (with the exception of `Value::Null`).
Instead, you should typically use `Value::from` on the type you wish to convert. See the
[documentation](https://docs.rs/bolt-proto/*/bolt_proto/value/enum.Value.html) for more details.
