pub use self::client::Client;

mod client;
mod error;
mod message;
mod serialize;
mod structure;
mod utils;
mod value;

// TODO: Maybe use tokio-tower to build the protocol instead of manually encoding/decoding everything
