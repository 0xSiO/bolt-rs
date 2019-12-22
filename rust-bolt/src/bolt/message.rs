pub use bolt::init::BoltInit;
pub use bolt::success::BoltSuccess;
pub use chunk::Chunk;
pub use message_bytes::BoltMessageBytes;

mod bolt;
mod chunk;
mod message_bytes;

#[derive(Debug)]
pub enum BoltMessage {
    Init(BoltInit),
    Success(BoltSuccess),
}
