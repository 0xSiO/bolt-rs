pub use self::client::Client;

mod bolt;
mod client;
mod error;
mod serialize;
mod structure;

// TODO: Maybe use tokio-tower to build the protocol instead of manually encoding/decoding everything
