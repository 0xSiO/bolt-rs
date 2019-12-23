pub use self::client::Client;

mod bolt;
mod client;
mod error;
mod native;
mod serialize;

// TODO: Maybe use tokio-tower to build the protocol instead of manually encoding/decoding everything
