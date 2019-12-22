pub use bolt::init::BoltInit;
pub use bolt::success::BoltSuccess;
pub use chunk::Chunk;
pub use message_bytes::MessageBytes;

mod bolt;
mod chunk;
mod message_bytes;

#[derive(Debug)]
pub enum Message {
    Init(BoltInit),
    Success(BoltSuccess),
}
