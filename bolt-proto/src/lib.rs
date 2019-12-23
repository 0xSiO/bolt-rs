pub use serialize::{Deserialize, Serialize};

pub mod bolt;
pub mod error;
pub mod native;
mod serialize;

// TODO: Maybe use tokio-tower to build the protocol instead of manually encoding/decoding everything
