pub use chunk::Chunk;
pub use message_bytes::MessageBytes;
pub use raw::init::InitRaw;
pub use raw::success::SuccessRaw;

mod chunk;
mod message_bytes;
mod raw;

#[derive(Debug)]
pub enum Message {
    Init(InitRaw),
    Success(SuccessRaw),
}
