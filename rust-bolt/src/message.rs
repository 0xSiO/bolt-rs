pub use chunk::Chunk;
pub use init::InitRaw;
pub use message_bytes::MessageBytes;
pub use success::SuccessRaw;

mod chunk;
mod init;
mod message_bytes;
mod success;

#[derive(Debug)]
pub enum Message {
    Init(InitRaw),
    Success(SuccessRaw),
}
