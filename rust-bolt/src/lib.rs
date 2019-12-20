pub use self::client::Client;

mod client;
mod message;
mod serialize;
mod value;

// TODO: Maybe use tokio-proto to build the protocol instead of manually encoding/decoding everything
